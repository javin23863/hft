use anyhow::Result;
use mempool_silver::{build_feature_bars, read_snapshots_parquet, write_features_parquet};

fn main() -> Result<()> {
    let in_path = "runtime/spool/hft/bronze/source=bitcoin/dataset=mempool_fee_snapshot/run=latest/snapshot.parquet";
    let out_path =
        "runtime/spool/hft/silver/source=bitcoin/dataset=mempool_features_1m/run=latest/features.parquet";
    let snaps = read_snapshots_parquet(in_path)?;
    let bars = build_feature_bars(&snaps);
    write_features_parquet(out_path, &bars)?;
    println!("wrote {}", out_path);
    Ok(())
}
