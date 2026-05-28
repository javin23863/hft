use anyhow::Result;
use feasibility::{
    build_report, write_report_markdown, H1Observation, H2Observation, H3Observation,
};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn read_jsonl<T: for<'de> serde::Deserialize<'de>>(path: impl AsRef<Path>) -> Result<Vec<T>> {
    let mut out = Vec::new();
    let file = File::open(path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line)?);
    }
    Ok(out)
}

fn main() -> Result<()> {
    let h1_path = std::env::var("HFT_H1_INPUT")
        .unwrap_or_else(|_| "data/feasibility/h1_observations.jsonl".to_string());
    let h2_path = std::env::var("HFT_H2_INPUT")
        .unwrap_or_else(|_| "data/feasibility/h2_observations.jsonl".to_string());
    let h3_path = std::env::var("HFT_H3_INPUT")
        .unwrap_or_else(|_| "data/feasibility/h3_observations.jsonl".to_string());

    let h1: Vec<H1Observation> = read_jsonl(h1_path)?;
    let h2: Vec<H2Observation> = read_jsonl(h2_path)?;
    let h3: Vec<H3Observation> = read_jsonl(h3_path)?;
    let report = build_report(&h1, &h2, &h3);
    write_report_markdown("docs/FEASIBILITY_REPORT.md", &report)?;
    println!("wrote docs/FEASIBILITY_REPORT.md");
    Ok(())
}
