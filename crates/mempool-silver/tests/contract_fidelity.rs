use std::fs;

use arrow_array::RecordBatchReader;
use jsonschema::JSONSchema;
use mempool_core::typed_parquet::{
    read_fee_snapshots_typed, write_feature_bars_typed, write_fee_snapshots_typed,
    write_scenario_signals_typed,
};
use mempool_core::{FeeSnapshot, ScenarioSignal, SCHEMA_VERSION};
use mempool_silver::{build_feature_bars, load_registry};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use scenario_engines::{emit_signals_for_feature, write_signals_parquet};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

fn load_schema(path: std::path::PathBuf) -> serde_json::Value {
    let content = fs::read_to_string(path).expect("read schema");
    serde_json::from_str(&content).expect("parse schema")
}

fn validate_schema(schema: &serde_json::Value, row: &serde_json::Value) {
    let compiled = JSONSchema::compile(schema).expect("compile schema");
    let validation = compiled.validate(row);
    if let Err(errors) = validation {
        let msg = errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        panic!("schema validation failed: {msg}");
    }
}

fn digest_json_rows(rows: &[serde_json::Value]) -> String {
    let mut hasher = Sha256::new();
    for row in rows {
        hasher.update(serde_json::to_vec(row).expect("serialize json row"));
    }
    format!("{:x}", hasher.finalize())
}

fn parquet_has_typed_columns(path: &std::path::Path, expected: &[&str]) {
    let file = fs::File::open(path).unwrap();
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .unwrap()
        .build()
        .unwrap();
    let schema = reader.schema();
    for col in expected {
        assert!(
            schema.field_with_name(col).is_ok(),
            "missing typed column {col}"
        );
    }
}

#[test]
fn typed_parquet_roundtrip_validates_schema_and_hashes() {
    let tmp = tempdir().expect("tempdir");
    let fee_path = tmp.path().join("fees.parquet");
    let feature_path = tmp.path().join("features.parquet");
    let signal_path = tmp.path().join("signals.parquet");

    let schema_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas/v1");
    let fee_schema = load_schema(schema_root.join("mempool_fee_snapshot.schema.json"));
    let signal_schema = load_schema(schema_root.join("scenario_signal.schema.json"));

    let snaps = vec![
        FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:00:00Z".into(),
            size_txs: 100,
            bytes: 1_000_000,
            min_fee_sat_vb: 2.0,
            p50_fee_sat_vb: 5.0,
            p90_fee_sat_vb: 15.0,
            p99_fee_sat_vb: 35.0,
            stuck_flow_pct: 12.0,
            source_node_id: "node-a".into(),
        },
        FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:01:00Z".into(),
            size_txs: 400,
            bytes: 4_000_000,
            min_fee_sat_vb: 3.0,
            p50_fee_sat_vb: 9.0,
            p90_fee_sat_vb: 32.0,
            p99_fee_sat_vb: 90.0,
            stuck_flow_pct: 82.0,
            source_node_id: "node-a".into(),
        },
    ];

    write_fee_snapshots_typed(&fee_path, &snaps).expect("write fee parquet");
    parquet_has_typed_columns(
        &fee_path,
        &["observed_at", "p99_fee_sat_vb", "stuck_flow_pct"],
    );
    let snap_back = read_fee_snapshots_typed(&fee_path).expect("read fee parquet");
    assert_eq!(snap_back.len(), snaps.len());
    let fee_json = snap_back
        .iter()
        .map(|x| serde_json::to_value(x).expect("fee to value"))
        .collect::<Vec<_>>();
    for row in &fee_json {
        validate_schema(&fee_schema, row);
    }

    let registry = load_registry().unwrap_or_else(|_| {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ex.json");
        fs::write(
            &path,
            r#"{"exchanges":[{"name":"t","addresses":["bc1qdep"]}]}"#,
        )
        .unwrap();
        mempool_core::ExchangeRegistry::load(path).unwrap()
    });
    let mut bars = build_feature_bars(&snap_back, &[], &registry, 10.0);
    bars[1].cpfp_detected = true;
    bars[1].exchange_inflow_event = true;
    write_feature_bars_typed(&feature_path, &bars).expect("write features parquet");
    parquet_has_typed_columns(
        &feature_path,
        &["fee_spike_zscore", "exchange_inflow_event", "cpfp_detected"],
    );

    let signals: Vec<ScenarioSignal> = bars
        .iter()
        .flat_map(emit_signals_for_feature)
        .collect();
    write_signals_parquet(&signal_path, &signals).expect("write signals parquet");
    write_scenario_signals_typed(&signal_path, &signals).expect("write typed signals");

    let signal_json = signals
        .iter()
        .map(|x| serde_json::to_value(x).expect("signal to value"))
        .collect::<Vec<_>>();
    for row in &signal_json {
        validate_schema(&signal_schema, row);
    }

    let hash_a = digest_json_rows(&signal_json);
    let hash_b = digest_json_rows(&signal_json);
    assert_eq!(hash_a, hash_b, "deterministic row hash mismatch");
}
