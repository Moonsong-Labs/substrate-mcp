use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

/// Query runtime upgrades from historical blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUpgradeQuery {
    /// Start block (negative = relative to current)
    pub from_block: i32,
    /// End block (negative = relative to current)  
    pub to_block: Option<i32>,
}

/// Result of runtime upgrade query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUpgradeResult {
    /// Runtime upgrades found
    pub upgrades: Vec<RuntimeUpgrade>,
    /// Number of blocks queried
    pub blocks_queried: u32,
    /// Current block height
    pub current_block: u32,
}

/// A runtime upgrade occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUpgrade {
    /// Block number where upgrade occurred
    pub block_number: u32,
    /// Block hash
    pub block_hash: String,
    /// Previous spec version
    pub prev_spec_version: u32,
    /// New spec version
    pub new_spec_version: u32,
    /// Previous spec name
    pub prev_spec_name: String,
    /// New spec name
    pub new_spec_name: String,
    /// Code hash of the new runtime
    pub code_hash: String,
}

/// Detailed runtime upgrade information with associated events, storage changes, and transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUpgradeDetails {
    /// The runtime upgrade info
    pub upgrade: RuntimeUpgrade,
    /// Events in the upgrade block
    pub events: Vec<crate::substrate::historical::HistoricalEvent>,
    /// Storage changes in the upgrade block
    pub storage_changes: Vec<StorageChange>,
    /// Transactions in the upgrade block
    pub transactions: Vec<crate::substrate::transactions::HistoricalTransaction>,
}

/// A storage change in the upgrade block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChange {
    /// Storage key
    pub key: String,
    /// Previous value (if any)
    pub prev_value: Option<String>,
    /// New value (if any)
    pub new_value: Option<String>,
    /// Pallet name (if determinable)
    pub pallet: Option<String>,
    /// Storage item name (if determinable)
    pub storage_item: Option<String>,
}

/// Query for runtime upgrades in a block range
pub async fn query_runtime_upgrades(
    query: RuntimeUpgradeQuery,
    _subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<RuntimeUpgradeResult> {
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::ws_client::WsClientBuilder;

    // Create WebSocket RPC client
    let rpc_client = WsClientBuilder::default().build(rpc_url).await?;

    // Get current block number
    let current_block: u32 = {
        let params: Vec<serde_json::Value> = vec![];
        let header: serde_json::Value = rpc_client.request("chain_getHeader", params).await?;

        let number_hex = header["number"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No block number in header"))?;

        u32::from_str_radix(&number_hex[2..], 16)?
    };

    // Calculate actual block range
    let from = if query.from_block < 0 {
        (current_block as i32 + query.from_block) as u32
    } else {
        query.from_block as u32
    };

    let to = match query.to_block {
        Some(b) if b < 0 => (current_block as i32 + b) as u32,
        Some(b) => b as u32,
        None => current_block,
    };

    let mut upgrades = Vec::new();
    let blocks_queried = to - from + 1;

    // Storage key for System.LastRuntimeUpgrade
    let _last_runtime_upgrade_key = "0x26aa394eea5630e07c48ae0c9558cef7f9cce9c888469bb1a0dceaa129672ef8";

    let mut prev_runtime_info: Option<(u32, String)> = None;

    // Query each block
    for block_num in from..=to {
        // Get block hash
        let block_hash: Option<String> = rpc_client
            .request("chain_getBlockHash", vec![block_num])
            .await?;

        let block_hash = block_hash.ok_or_else(|| anyhow::anyhow!("Block {} not found", block_num))?;

        // Get runtime version at this block
        let runtime_version: serde_json::Value = rpc_client
            .request("state_getRuntimeVersion", vec![&block_hash])
            .await?;

        let spec_version = runtime_version["specVersion"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("No spec version"))? as u32;

        let spec_name = runtime_version["specName"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // Check if this is an upgrade
        if let Some((prev_version, prev_name)) = &prev_runtime_info {
            if *prev_version != spec_version {
                // Get the code hash
                let code_storage: Option<String> = rpc_client
                    .request("state_getStorage", ("0x3a636f6465", &block_hash))
                    .await?;

                let code_hash = if let Some(code) = code_storage {
                    let code_bytes = hex::decode(code.trim_start_matches("0x"))?;
                    // Create a simple hash from the first 32 bytes
                    if code_bytes.len() >= 32 {
                        format!("0x{}", hex::encode(&code_bytes[..32]))
                    } else {
                        // Pad with zeros if less than 32 bytes
                        let mut padded = vec![0u8; 32];
                        padded[..code_bytes.len()].copy_from_slice(&code_bytes);
                        format!("0x{}", hex::encode(padded))
                    }
                } else {
                    "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
                };

                upgrades.push(RuntimeUpgrade {
                    block_number: block_num,
                    block_hash: block_hash.clone(),
                    prev_spec_version: *prev_version,
                    new_spec_version: spec_version,
                    prev_spec_name: prev_name.clone(),
                    new_spec_name: spec_name.clone(),
                    code_hash,
                });
            }
        }

        prev_runtime_info = Some((spec_version, spec_name));
    }

    Ok(RuntimeUpgradeResult {
        upgrades,
        blocks_queried,
        current_block,
    })
}

/// Search for a specific runtime upgrade and get detailed information
pub async fn search_runtime_upgrade(
    from_block: i32,
    to_block: Option<i32>,
    target_spec_version: Option<u32>,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<Option<RuntimeUpgradeDetails>> {
    // First, find runtime upgrades in the range
    let query = RuntimeUpgradeQuery {
        from_block,
        to_block,
    };

    let upgrades_result = query_runtime_upgrades(query, subxt_client, rpc_url).await?;

    // Find the matching upgrade
    let upgrade = if let Some(target_version) = target_spec_version {
        upgrades_result
            .upgrades
            .into_iter()
            .find(|u| u.new_spec_version == target_version)
    } else {
        // If no specific version requested, return the first upgrade found
        upgrades_result.upgrades.into_iter().next()
    };

    if let Some(upgrade) = upgrade {
        // Get all events in the upgrade block
        let events_query = crate::substrate::historical::HistoricalEventsQuery {
            from_block: upgrade.block_number as i32,
            to_block: Some(upgrade.block_number as i32),
            pallet: None,
            event: None,
        };

        let events = crate::substrate::historical::query_historical_events(
            events_query,
            subxt_client,
            rpc_url,
        )
        .await?;

        // Get all transactions in the upgrade block
        let tx_query = crate::substrate::transactions::HistoricalTransactionsQuery {
            from_block: upgrade.block_number as i32,
            to_block: Some(upgrade.block_number as i32),
            pallet: None,
            call: None,
            signer: None,
        };

        let transactions = crate::substrate::transactions::query_historical_transactions(
            tx_query,
            subxt_client,
            rpc_url,
        )
        .await?;

        // Get storage changes (focusing on important system storage)
        let storage_changes = get_upgrade_storage_changes(
            &upgrade.block_hash,
            upgrade.block_number,
            rpc_url,
        )
        .await?;

        Ok(Some(RuntimeUpgradeDetails {
            upgrade,
            events: events.events,
            storage_changes,
            transactions: transactions.transactions,
        }))
    } else {
        Ok(None)
    }
}

/// Get storage changes for important system keys during upgrade
async fn get_upgrade_storage_changes(
    block_hash: &str,
    block_number: u32,
    rpc_url: &str,
) -> Result<Vec<StorageChange>> {
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::ws_client::WsClientBuilder;

    let rpc_client = WsClientBuilder::default().build(rpc_url).await?;

    // Get the parent block hash
    let parent_hash: Option<String> = rpc_client
        .request("chain_getBlockHash", vec![block_number - 1])
        .await?;

    let parent_hash = parent_hash.ok_or_else(|| anyhow::anyhow!("Parent block not found"))?;

    let mut changes = Vec::new();

    // Important storage keys to check
    let important_keys = vec![
        ("0x26aa394eea5630e07c48ae0c9558cef7f9cce9c888469bb1a0dceaa129672ef8", "System", "LastRuntimeUpgrade"),
        ("0x3a636f6465", "System", ":code"),
        ("0x26aa394eea5630e07c48ae0c9558cef79a5f0fe3a994afd1160bf61dd10b857e66", "System", "UpgradedToU32RefCount"),
        ("0x26aa394eea5630e07c48ae0c9558cef7682a096bba730d67f8488aa00d6bcea6", "System", "UpgradedToTripleRefCount"),
    ];

    for (key, pallet, item) in important_keys {
        // Get value at parent block
        let prev_value: Option<String> = rpc_client
            .request("state_getStorage", (key, &parent_hash))
            .await?;

        // Get value at current block
        let new_value: Option<String> = rpc_client
            .request("state_getStorage", (key, block_hash))
            .await?;

        // Only record if there was a change
        if prev_value != new_value {
            changes.push(StorageChange {
                key: key.to_string(),
                prev_value,
                new_value,
                pallet: Some(pallet.to_string()),
                storage_item: Some(item.to_string()),
            });
        }
    }

    Ok(changes)
}