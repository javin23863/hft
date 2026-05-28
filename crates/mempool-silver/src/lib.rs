pub mod backtest;
pub mod feature_adapter;

pub use backtest::{BacktestSummary, run_regime_backtest};
pub use feature_adapter::{
    build_feature_bars, load_registry, read_snapshots_from_run_dir, read_snapshots_jsonl,
    read_snapshots_parquet, read_tx_events_from_run_dir, read_tx_events_jsonl,
    write_features_jsonl, write_features_parquet,
};
