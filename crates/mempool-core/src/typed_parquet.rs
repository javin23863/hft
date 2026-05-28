use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use arrow_array::{ArrayRef, BooleanArray, Float64Array, RecordBatch, StringArray, UInt64Array};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;

use crate::types::{FeeSnapshot, MempoolFeatureBar, ScenarioSignal};

fn fee_snapshot_schema() -> Schema {
    Schema::new(vec![
        Field::new("schema_version", DataType::Utf8, false),
        Field::new("observed_at", DataType::Utf8, false),
        Field::new("size_txs", DataType::UInt64, false),
        Field::new("bytes", DataType::UInt64, false),
        Field::new("min_fee_sat_vb", DataType::Float64, false),
        Field::new("p50_fee_sat_vb", DataType::Float64, false),
        Field::new("p90_fee_sat_vb", DataType::Float64, false),
        Field::new("p99_fee_sat_vb", DataType::Float64, false),
        Field::new("stuck_flow_pct", DataType::Float64, false),
        Field::new("source_node_id", DataType::Utf8, false),
    ])
}

fn feature_bar_schema() -> Schema {
    Schema::new(vec![
        Field::new("schema_version", DataType::Utf8, false),
        Field::new("ts_minute", DataType::Utf8, false),
        Field::new("tx_count", DataType::UInt64, false),
        Field::new("avg_fee_sat_vb", DataType::Float64, false),
        Field::new("p90_fee_sat_vb", DataType::Float64, false),
        Field::new("p99_fee_sat_vb", DataType::Float64, false),
        Field::new("stuck_flow_pct", DataType::Float64, false),
        Field::new("fee_spike_zscore", DataType::Float64, false),
        Field::new("exchange_inflow_event", DataType::Boolean, false),
        Field::new("cpfp_detected", DataType::Boolean, false),
        Field::new("congestion_regime", DataType::Utf8, false),
    ])
}

fn write_batch(path: impl AsRef<Path>, schema: Schema, batch: RecordBatch) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create parquet parent")?;
    }
    let file = File::create(path).context("create parquet file")?;
    let mut writer = ArrowWriter::try_new(file, Arc::new(schema), None).context("arrow writer")?;
    writer.write(&batch).context("write batch")?;
    writer.close().context("close parquet")?;
    Ok(())
}

fn read_batches(path: impl AsRef<Path>) -> Result<Vec<RecordBatch>> {
    let file = File::open(path.as_ref()).context("open parquet")?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .context("parquet reader builder")?
        .build()
        .context("build parquet reader")?;
    reader.collect::<Result<Vec<_>, _>>().context("read batches")
}

pub fn write_fee_snapshots_typed(path: impl AsRef<Path>, rows: &[FeeSnapshot]) -> Result<()> {
    let schema = fee_snapshot_schema();
    let str_col = |f: fn(&FeeSnapshot) -> &str| -> ArrayRef {
        Arc::new(StringArray::from(
            rows.iter().map(f).collect::<Vec<_>>(),
        )) as ArrayRef
    };
    let u64_col = |f: fn(&FeeSnapshot) -> u64| -> ArrayRef {
        Arc::new(UInt64Array::from(rows.iter().map(f).collect::<Vec<_>>())) as ArrayRef
    };
    let f64_col = |f: fn(&FeeSnapshot) -> f64| -> ArrayRef {
        Arc::new(Float64Array::from(rows.iter().map(f).collect::<Vec<_>>())) as ArrayRef
    };
    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            str_col(|r| &r.schema_version),
            str_col(|r| &r.observed_at),
            u64_col(|r| r.size_txs),
            u64_col(|r| r.bytes),
            f64_col(|r| r.min_fee_sat_vb),
            f64_col(|r| r.p50_fee_sat_vb),
            f64_col(|r| r.p90_fee_sat_vb),
            f64_col(|r| r.p99_fee_sat_vb),
            f64_col(|r| r.stuck_flow_pct),
            str_col(|r| &r.source_node_id),
        ],
    )
    .context("fee snapshot batch")?;
    write_batch(path, schema, batch)
}

pub fn read_fee_snapshots_typed(path: impl AsRef<Path>) -> Result<Vec<FeeSnapshot>> {
    let mut out = Vec::new();
    for batch in read_batches(path)? {
        let schema_version = batch
            .column_by_name("schema_version")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("schema_version col")?;
        let observed_at = batch
            .column_by_name("observed_at")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("observed_at col")?;
        let size_txs = batch
            .column_by_name("size_txs")
            .and_then(|c| c.as_any().downcast_ref::<UInt64Array>())
            .context("size_txs col")?;
        let bytes = batch
            .column_by_name("bytes")
            .and_then(|c| c.as_any().downcast_ref::<UInt64Array>())
            .context("bytes col")?;
        let min_fee = batch
            .column_by_name("min_fee_sat_vb")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("min_fee col")?;
        let p50 = batch
            .column_by_name("p50_fee_sat_vb")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("p50 col")?;
        let p90 = batch
            .column_by_name("p90_fee_sat_vb")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("p90 col")?;
        let p99 = batch
            .column_by_name("p99_fee_sat_vb")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("p99 col")?;
        let stuck = batch
            .column_by_name("stuck_flow_pct")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("stuck col")?;
        let node = batch
            .column_by_name("source_node_id")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("source_node_id col")?;
        for i in 0..batch.num_rows() {
            out.push(FeeSnapshot {
                schema_version: schema_version.value(i).to_string(),
                observed_at: observed_at.value(i).to_string(),
                size_txs: size_txs.value(i),
                bytes: bytes.value(i),
                min_fee_sat_vb: min_fee.value(i),
                p50_fee_sat_vb: p50.value(i),
                p90_fee_sat_vb: p90.value(i),
                p99_fee_sat_vb: p99.value(i),
                stuck_flow_pct: stuck.value(i),
                source_node_id: node.value(i).to_string(),
            });
        }
    }
    Ok(out)
}

pub fn write_feature_bars_typed(path: impl AsRef<Path>, rows: &[MempoolFeatureBar]) -> Result<()> {
    let schema = feature_bar_schema();
    let str_col = |f: fn(&MempoolFeatureBar) -> &str| -> ArrayRef {
        Arc::new(StringArray::from(rows.iter().map(f).collect::<Vec<_>>())) as ArrayRef
    };
    let u64_col = |f: fn(&MempoolFeatureBar) -> u64| -> ArrayRef {
        Arc::new(UInt64Array::from(rows.iter().map(f).collect::<Vec<_>>())) as ArrayRef
    };
    let f64_col = |f: fn(&MempoolFeatureBar) -> f64| -> ArrayRef {
        Arc::new(Float64Array::from(rows.iter().map(f).collect::<Vec<_>>())) as ArrayRef
    };
    let bool_col = |f: fn(&MempoolFeatureBar) -> bool| -> ArrayRef {
        Arc::new(BooleanArray::from(rows.iter().map(f).collect::<Vec<_>>())) as ArrayRef
    };
    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            str_col(|r| &r.schema_version),
            str_col(|r| &r.ts_minute),
            u64_col(|r| r.tx_count),
            f64_col(|r| r.avg_fee_sat_vb),
            f64_col(|r| r.p90_fee_sat_vb),
            f64_col(|r| r.p99_fee_sat_vb),
            f64_col(|r| r.stuck_flow_pct),
            f64_col(|r| r.fee_spike_zscore),
            bool_col(|r| r.exchange_inflow_event),
            bool_col(|r| r.cpfp_detected),
            str_col(|r| &r.congestion_regime),
        ],
    )
    .context("feature bar batch")?;
    write_batch(path, schema, batch)
}

pub fn read_feature_bars_typed(path: impl AsRef<Path>) -> Result<Vec<MempoolFeatureBar>> {
    let mut out = Vec::new();
    for batch in read_batches(path)? {
        let schema_version = batch
            .column_by_name("schema_version")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("schema_version")?;
        let ts_minute = batch
            .column_by_name("ts_minute")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("ts_minute")?;
        let tx_count = batch
            .column_by_name("tx_count")
            .and_then(|c| c.as_any().downcast_ref::<UInt64Array>())
            .context("tx_count")?;
        let avg_fee = batch
            .column_by_name("avg_fee_sat_vb")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("avg_fee")?;
        let p90 = batch
            .column_by_name("p90_fee_sat_vb")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("p90")?;
        let p99 = batch
            .column_by_name("p99_fee_sat_vb")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("p99")?;
        let stuck = batch
            .column_by_name("stuck_flow_pct")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("stuck")?;
        let z = batch
            .column_by_name("fee_spike_zscore")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("z")?;
        let inflow = batch
            .column_by_name("exchange_inflow_event")
            .and_then(|c| c.as_any().downcast_ref::<BooleanArray>())
            .context("inflow")?;
        let cpfp = batch
            .column_by_name("cpfp_detected")
            .and_then(|c| c.as_any().downcast_ref::<BooleanArray>())
            .context("cpfp")?;
        let regime = batch
            .column_by_name("congestion_regime")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("congestion_regime")?;
        for i in 0..batch.num_rows() {
            out.push(MempoolFeatureBar {
                schema_version: schema_version.value(i).to_string(),
                ts_minute: ts_minute.value(i).to_string(),
                tx_count: tx_count.value(i),
                avg_fee_sat_vb: avg_fee.value(i),
                p90_fee_sat_vb: p90.value(i),
                p99_fee_sat_vb: p99.value(i),
                stuck_flow_pct: stuck.value(i),
                fee_spike_zscore: z.value(i),
                exchange_inflow_event: inflow.value(i),
                cpfp_detected: cpfp.value(i),
                congestion_regime: regime.value(i).to_string(),
            });
        }
    }
    Ok(out)
}

/// Typed scenario_signal parquet (posterior fields flattened).
pub fn write_scenario_signals_typed(path: impl AsRef<Path>, rows: &[ScenarioSignal]) -> Result<()> {
    let schema = Schema::new(vec![
        Field::new("schema_version", DataType::Utf8, false),
        Field::new("observed_at", DataType::Utf8, false),
        Field::new("hypothesis_id", DataType::Utf8, false),
        Field::new("signal_name", DataType::Utf8, false),
        Field::new("confidence", DataType::Float64, false),
        Field::new("posterior_mean", DataType::Float64, false),
        Field::new("posterior_std", DataType::Float64, false),
        Field::new("posterior_n_obs", DataType::UInt64, false),
        Field::new("posterior_method", DataType::Utf8, false),
        Field::new("payload_json", DataType::Utf8, false),
    ]);
    let payload_json: Vec<String> = rows.iter().map(|r| r.payload.to_string()).collect();
    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(StringArray::from(
                rows.iter().map(|r| r.schema_version.as_str()).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                rows.iter().map(|r| r.observed_at.as_str()).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|r| r.hypothesis_id.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                rows.iter().map(|r| r.signal_name.as_str()).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Float64Array::from(
                rows.iter().map(|r| r.confidence).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Float64Array::from(
                rows.iter().map(|r| r.posterior.mean).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Float64Array::from(
                rows.iter().map(|r| r.posterior.std).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(UInt64Array::from(
                rows.iter().map(|r| r.posterior.n_obs).collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|r| r.posterior.method.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                payload_json.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )) as ArrayRef,
        ],
    )
    .context("scenario signal batch")?;
    write_batch(path, schema, batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SCHEMA_VERSION;
    use tempfile::tempdir;

    #[test]
    fn fee_snapshot_typed_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fees.parquet");
        let rows = vec![FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:00:00Z".into(),
            size_txs: 42,
            bytes: 9000,
            min_fee_sat_vb: 1.5,
            p50_fee_sat_vb: 3.0,
            p90_fee_sat_vb: 8.0,
            p99_fee_sat_vb: 15.0,
            stuck_flow_pct: 22.0,
            source_node_id: "node-1".into(),
        }];
        write_fee_snapshots_typed(&path, &rows).unwrap();
        let back = read_fee_snapshots_typed(&path).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].size_txs, 42);
        assert!((back[0].p99_fee_sat_vb - 15.0).abs() < 1e-9);
    }
}
