//! Shared types, schema version, and lake path helpers for HFT mempool ingest.

pub mod config;
pub mod lake;
pub mod parquet_io;
pub mod schema;
pub mod types;

pub use config::IngestConfig;
pub use lake::{
    bronze_fee_snapshot_key, bronze_tx_event_key, silver_features_1m_key,
    silver_scenario_signal_key,
};
pub use schema::SCHEMA_VERSION;
pub use types::{
    FeeSnapshot, MempoolEntryMeta, MempoolFeatureBar, MempoolTxEvent, ScenarioSignal,
    SignalPosteriorFields,
};
