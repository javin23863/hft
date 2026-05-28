use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct RegistryFile {
    exchanges: Vec<ExchangeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExchangeEntry {
    #[allow(dead_code)]
    name: String,
    addresses: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExchangeRegistry {
    addresses: HashSet<String>,
}

impl ExchangeRegistry {
    pub fn from_addresses(addresses: HashSet<String>) -> Self {
        Self { addresses }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut addresses = HashSet::new();
        merge_registry_file(path, &mut addresses)?;
        if let Some(parent) = path.parent() {
            let deposits = parent.join("labeled_deposits.json");
            merge_registry_file(&deposits, &mut addresses).ok();
        }
        Ok(Self { addresses })
    }

    pub fn contains_address(&self, address: &str) -> bool {
        self.addresses.contains(address)
    }

    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }

    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    pub fn tx_has_exchange_output<'a, I>(&self, outputs: I) -> bool
    where
        I: IntoIterator<Item = &'a Option<String>>,
    {
        outputs.into_iter().flatten().any(|a| self.contains_address(a))
    }
}

fn merge_registry_file(path: &Path, out: &mut HashSet<String>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let parsed: RegistryFile = serde_json::from_str(&content).context("parse exchange registry")?;
    for ex in parsed.exchanges {
        for addr in ex.addresses {
            if !addr.is_empty() {
                out.insert(addr);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_configured_address() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("exchanges.json");
        fs::write(
            &path,
            r#"{"exchanges":[{"name":"test","addresses":["bc1qtest"]}]}"#,
        )
        .unwrap();
        let reg = ExchangeRegistry::load(&path).unwrap();
        assert!(reg.contains_address("bc1qtest"));
        assert!(!reg.contains_address("bc1qother"));
    }

    #[test]
    fn merges_labeled_deposits_overlay() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("exchanges.json");
        fs::write(
            &path,
            r#"{"exchanges":[{"name":"base","addresses":["bc1qbase"]}]}"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("labeled_deposits.json"),
            r#"{"exchanges":[{"name":"overlay","addresses":["bc1qoverlay"]}]}"#,
        )
        .unwrap();
        let reg = ExchangeRegistry::load(&path).unwrap();
        assert_eq!(reg.len(), 2);
        assert!(reg.contains_address("bc1qoverlay"));
    }
}
