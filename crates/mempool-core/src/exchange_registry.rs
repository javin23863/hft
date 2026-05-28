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
        let content = fs::read_to_string(path.as_ref()).context("read exchange registry")?;
        let parsed: RegistryFile =
            serde_json::from_str(&content).context("parse exchange registry")?;
        let mut addresses = HashSet::new();
        for ex in parsed.exchanges {
            for addr in ex.addresses {
                if !addr.is_empty() {
                    addresses.insert(addr);
                }
            }
        }
        Ok(Self { addresses })
    }

    pub fn contains_address(&self, address: &str) -> bool {
        self.addresses.contains(address)
    }

    pub fn tx_has_exchange_output<'a, I>(&self, outputs: I) -> bool
    where
        I: IntoIterator<Item = &'a Option<String>>,
    {
        outputs.into_iter().flatten().any(|a| self.contains_address(a))
    }
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
}
