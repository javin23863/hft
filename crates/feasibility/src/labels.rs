use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct TxLabelRow {
    txid: String,
    actual_positive: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GroundTruthLabels {
    pub h1: HashMap<String, bool>,
    pub h2: HashMap<String, bool>,
}

impl GroundTruthLabels {
    pub fn load_default() -> Result<Self> {
        Self::load_dir("data/feasibility/ground_truth")
    }

    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let mut labels = Self::default();
        if !dir.exists() {
            return Ok(labels);
        }
        labels.merge_jsonl(&dir.join("h1_labels.jsonl"))?;
        labels.merge_jsonl(&dir.join("h2_labels.jsonl"))?;
        Ok(labels)
    }

    fn merge_jsonl(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let is_h2 = stem.contains("h2");
        for line in BufReader::new(file).lines() {
            let line = line.context("read label line")?;
            if line.trim().is_empty() {
                continue;
            }
            let row: TxLabelRow = serde_json::from_str(&line).context("parse label row")?;
            if is_h2 {
                self.h2.insert(row.txid, row.actual_positive);
            } else {
                self.h1.insert(row.txid, row.actual_positive);
            }
        }
        Ok(())
    }
}
