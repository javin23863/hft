use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use parquet::basic::{Compression, Repetition, Type as PhysicalType};
use parquet::column::writer::ColumnWriter;
use parquet::data_type::ByteArray;
use parquet::file::properties::WriterProperties;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::file::writer::SerializedFileWriter;
use parquet::record::RowAccessor;
use parquet::schema::types::Type;
use serde::de::DeserializeOwned;
use serde::Serialize;

fn json_schema() -> Result<Arc<Type>> {
    let field = Type::primitive_type_builder("json", PhysicalType::BYTE_ARRAY)
        .with_repetition(Repetition::REQUIRED)
        .build()
        .context("build parquet field schema")?;
    let schema = Type::group_type_builder("hft_row")
        .with_fields(vec![Arc::new(field)])
        .build()
        .context("build parquet group schema")?;
    Ok(Arc::new(schema))
}

pub fn write_json_rows_parquet<T: Serialize>(path: impl AsRef<Path>, rows: &[T]) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("create parquet parent dir")?;
    }

    let file = File::create(path).context("create parquet file")?;
    let schema = json_schema()?;
    let props = Arc::new(
        WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build(),
    );
    let mut writer =
        SerializedFileWriter::new(file, schema, props).context("open parquet writer")?;
    let mut row_group = writer.next_row_group().context("open row group")?;
    if let Some(mut col) = row_group.next_column().context("open first column")? {
        match col.untyped() {
            ColumnWriter::ByteArrayColumnWriter(typed) => {
                let encoded: Vec<ByteArray> = rows
                    .iter()
                    .map(|v| {
                        let json = serde_json::to_string(v).context("serialize parquet row json")?;
                        Ok(ByteArray::from(json.as_str()))
                    })
                    .collect::<Result<Vec<_>>>()?;
                typed
                    .write_batch(&encoded, None, None)
                    .context("write parquet batch")?;
            }
            _ => anyhow::bail!("unexpected parquet column writer type"),
        }
        col.close().context("close parquet column")?;
    }
    row_group.close().context("close parquet row group")?;
    writer.close().context("close parquet writer")?;
    Ok(())
}

pub fn read_json_rows_parquet<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<Vec<T>> {
    let file = File::open(path.as_ref()).context("open parquet file")?;
    let reader = SerializedFileReader::new(file).context("open parquet reader")?;
    let mut out = Vec::new();
    for row in reader.get_row_iter(None).context("create parquet row iter")? {
        let row = row.context("read parquet row")?;
        let bytes = row
            .get_bytes(0)
            .context("extract json bytes column from parquet row")?;
        let json = std::str::from_utf8(bytes.data()).context("decode json bytes as utf8")?;
        out.push(serde_json::from_str::<T>(json).context("deserialize parquet row json")?);
    }
    Ok(out)
}
