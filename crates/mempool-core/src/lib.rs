//! Shared types, schema version, and lake path helpers for HFT mempool ingest.

pub mod config;
pub mod exchange_registry;
pub mod features;
pub mod lake;
pub mod posterior;
pub mod schema;
pub mod typed_parquet;
pub mod types;

pub use config::IngestConfig;
pub use lake::{
    bronze_fee_snapshot_key, bronze_tx_event_key, silver_features_1m_key,
    silver_scenario_signal_key,
};
pub use exchange_registry::ExchangeRegistry;
pub use features::{
    aggregate_bar_flags, confirm_exchange_inflow, cpfp_consensus_truth, detect_cpfp,
    detect_exchange_inflow, has_enriched_outputs, predict_exchange_inflow,
};
pub use posterior::bootstrap_posterior;
pub use schema::SCHEMA_VERSION;
pub use typed_parquet::{
    read_feature_bars_typed, read_fee_snapshots_typed, write_feature_bars_typed,
    write_fee_snapshots_typed, write_scenario_signals_typed,
};
pub use types::{
    FeeSnapshot, MempoolEntryMeta, MempoolFeatureBar, MempoolTxEvent, ScenarioSignal,
    SignalPosteriorFields,
};
