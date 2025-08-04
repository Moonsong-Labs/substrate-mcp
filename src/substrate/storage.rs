use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::dynamic::{self, Value};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

/// Represents a storage entry value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    /// The pallet name
    pub pallet: String,
    /// The storage entry name
    pub entry: String,
    /// The storage key (hex encoded)
    pub key: String,
    /// The decoded value (if possible) or raw hex
    pub value: serde_json::Value,
    /// The block number where this value was queried
    pub at_block: Option<u32>,
}

/// Query parameters for storage entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuery {
    /// The pallet name
    pub pallet: String,
    /// The storage entry name
    pub entry: String,
    /// Optional keys for map-type storage (as JSON array)
    pub keys: Option<Vec<serde_json::Value>>,
    /// Block number to query at (None for latest)
    pub at_block: Option<u32>,
}

impl StorageQuery {
    /// Execute the storage query
    pub async fn execute(&self, client: &OnlineClient<PolkadotConfig>) -> Result<StorageEntry> {
        // Get metadata
        let metadata = client.metadata();

        // Find the pallet
        let pallet = metadata
            .pallet_by_name(&self.pallet)
            .ok_or_else(|| anyhow::anyhow!("Pallet '{}' not found", self.pallet))?;

        // Find the storage entry
        let storage = pallet
            .storage()
            .ok_or_else(|| anyhow::anyhow!("Pallet '{}' has no storage", self.pallet))?;

        let _entry = storage.entry_by_name(&self.entry).ok_or_else(|| {
            anyhow::anyhow!(
                "Storage entry '{}' not found in pallet '{}'",
                self.entry,
                self.pallet
            )
        })?;

        // Build the storage address dynamically
        let storage_address = dynamic::storage(&self.pallet, &self.entry, self.build_keys()?);

        // Get the block to query
        let block = if let Some(block_num) = self.at_block {
            // For specific block number, we need to get the hash
            // This is a simplified approach - in production, use proper RPC methods
            let latest = client.blocks().at_latest().await?;
            let hash = if block_num == latest.number() {
                latest.hash()
            } else {
                // For historical blocks, we'd need to use RPC methods
                return Err(anyhow::anyhow!(
                    "Historical block query not yet implemented for block {}",
                    block_num
                ));
            };
            client.blocks().at(hash).await?
        } else {
            client.blocks().at_latest().await?
        };

        // Fetch the storage value
        let result = block.storage().fetch(&storage_address).await?;

        // Format the result
        let value = match result {
            Some(storage_value) => {
                // Storage values are decoded by subxt using metadata
                // Try to decode to a dynamic Value
                match storage_value.to_value() {
                    Ok(decoded_value) => {
                        serde_json::json!({
                            "exists": true,
                            "decoded": format!("{:?}", decoded_value)
                        })
                    }
                    Err(e) => {
                        serde_json::json!({
                            "exists": true,
                            "error": format!("Failed to decode value: {}", e)
                        })
                    }
                }
            }
            None => serde_json::Value::Null,
        };

        // Get the storage key - for now just indicate the query params
        // A full implementation would encode the actual storage key
        let key = format!("{}.{}", self.pallet, self.entry);

        Ok(StorageEntry {
            pallet: self.pallet.clone(),
            entry: self.entry.clone(),
            key,
            value,
            at_block: self.at_block.or(Some(block.number())),
        })
    }

    /// Build keys for the storage query
    fn build_keys(&self) -> Result<Vec<Value>> {
        match &self.keys {
            Some(keys) => {
                // Convert JSON values to dynamic Values
                // This is a simplified implementation
                Ok(keys
                    .iter()
                    .map(|k| match k {
                        serde_json::Value::String(s) => Value::string(s),
                        serde_json::Value::Number(n) => {
                            if let Some(u) = n.as_u64() {
                                Value::u128(u as u128)
                            } else {
                                Value::i128(n.as_i64().unwrap_or(0) as i128)
                            }
                        }
                        serde_json::Value::Bool(b) => Value::bool(*b),
                        _ => Value::string(k.to_string()),
                    })
                    .collect())
            }
            None => Ok(vec![]),
        }
    }
}

/// Query multiple storage entries in a single batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStorageQuery {
    /// List of storage queries to execute
    pub queries: Vec<StorageQuery>,
}

impl BatchStorageQuery {
    /// Execute all storage queries
    #[allow(dead_code)]
    pub async fn execute(
        &self,
        client: &OnlineClient<PolkadotConfig>,
    ) -> Result<Vec<StorageEntry>> {
        let mut results = Vec::new();

        for query in &self.queries {
            match query.execute(client).await {
                Ok(entry) => results.push(entry),
                Err(e) => {
                    // Include error in result
                    results.push(StorageEntry {
                        pallet: query.pallet.clone(),
                        entry: query.entry.clone(),
                        key: String::new(),
                        value: serde_json::json!({
                            "error": e.to_string()
                        }),
                        at_block: query.at_block,
                    });
                }
            }
        }

        Ok(results)
    }
}

/// List all storage entries for a pallet
pub async fn list_pallet_storage(
    client: &OnlineClient<PolkadotConfig>,
    pallet_name: &str,
) -> Result<Vec<String>> {
    let metadata = client.metadata();

    let pallet = metadata
        .pallet_by_name(pallet_name)
        .ok_or_else(|| anyhow::anyhow!("Pallet '{}' not found", pallet_name))?;

    let storage = pallet
        .storage()
        .ok_or_else(|| anyhow::anyhow!("Pallet '{}' has no storage", pallet_name))?;

    Ok(storage
        .entries()
        .iter()
        .map(|e| e.name().to_string())
        .collect())
}
