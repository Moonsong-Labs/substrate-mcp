use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::dynamic::{self, Value};
use subxt::utils::AccountId32 as SubxtAccountId32;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

use super::utils;

/// Represents a storage entry value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
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
    /// Start block (negative = relative to current)
    pub from_block: i32,
    /// End block (negative = relative to current)  
    pub to_block: Option<i32>,
    /// The pallet name
    pub pallet: String,
    /// The storage entry name
    pub entry: String,
    /// Optional keys for map-type storage (as JSON array)
    pub keys: Option<Vec<serde_json::Value>>,
}

/// Result of storage query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult {
    /// Storage found
    pub storage: Vec<Storage>,
    /// Number of blocks queried
    pub blocks_queried: u32,
}

pub async fn query_storage(
    query: StorageQuery,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<StorageResult> {
    // Get block range from query parameters
    let (from, to) = utils::get_block_range(query.from_block, query.to_block, subxt_client).await?;

    // Get metadata
    let metadata = subxt_client.metadata();

    // Find the pallet
    let pallet = metadata
        .pallet_by_name(&query.pallet)
        .ok_or_else(|| anyhow::anyhow!("Pallet '{}' not found", query.pallet))?;

    // Find the storage entry
    let storage = pallet
        .storage()
        .ok_or_else(|| anyhow::anyhow!("Pallet '{}' has no storage", query.pallet))?;

    let _entry = storage.entry_by_name(&query.entry).ok_or_else(|| {
        anyhow::anyhow!(
            "Storage entry '{}' not found in pallet '{}'",
            query.entry,
            query.pallet
        )
    })?;

    // Build the storage address dynamically
    let storage_address = dynamic::storage(&query.pallet, &query.entry, build_keys(&query.keys)?);

    // Create RPC client for historical queries
    let rpc_client = utils::RpcClient::new(rpc_url).await?;

    let mut all_storages = Vec::new();
    let blocks_queried = to - from + 1;

    // Query each block
    for block_num in from..=to {
        // Get block hash
        let block_hash: Option<subxt::utils::H256> = rpc_client
            .request("chain_getBlockHash", vec![block_num])
            .await?;

        let block_hash =
            block_hash.ok_or_else(|| anyhow::anyhow!("Block {} not found", block_num))?;

        let block = subxt_client.blocks().at(block_hash).await?;

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
        let key = format!("{}.{}", query.pallet, query.entry);

        all_storages.push(Storage {
            pallet: query.pallet.clone(),
            entry: query.entry.clone(),
            key,
            value,
            at_block: Some(block_num),
        })
    }

    Ok(StorageResult {
        storage: all_storages,
        blocks_queried,
    })
}

/// Build keys for the storage query
fn build_keys(keys: &Option<Vec<serde_json::Value>>) -> Result<Vec<Value>> {
    match &keys {
        Some(keys) => {
            // Convert JSON values to dynamic Values
            Ok(keys
                .iter()
                .map(|k| match k {
                    serde_json::Value::String(s) => {
                        // Try to decode as SS58 address first
                        if let Ok(account_id) = s.parse::<SubxtAccountId32>() {
                            // Create a composite value with the AccountId32 bytes
                            let bytes: &[u8] = account_id.as_ref();
                            Value::from_bytes(bytes)
                        } else {
                            // If it's a String and not SS58, then convert directly
                            Value::string(s)
                        }
                    }
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
