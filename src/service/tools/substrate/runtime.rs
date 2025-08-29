use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

use super::events::{query_events, Event, EventsQuery};
use super::extrinsic::{query_extrinsics, Extrinsic, ExtrinsicsQuery};
use super::utils;

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
    pub events: Vec<Event>,
    /// Storage changes in the upgrade block
    pub storage_changes: Vec<StorageChange>,
    /// Transactions in the upgrade block
    pub transactions: Vec<Extrinsic>,
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

/// Runtime version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVersion {
    /// Spec version
    pub spec_version: u32,
    /// Spec name
    pub spec_name: String,
    /// Implementation name
    pub impl_name: String,
    /// Implementation version
    pub impl_version: u32,
    /// Authoring version
    pub authoring_version: u32,
    /// Transaction version
    pub transaction_version: u32,
    /// State version
    pub state_version: u32,
}

/// Query for runtime upgrades in a block range
pub async fn query_runtime_upgrades(
    query: RuntimeUpgradeQuery,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<RuntimeUpgradeResult> {
    // Get block range from query parameters
    let (from, to) = utils::get_block_range(query.from_block, query.to_block, subxt_client).await?;

    // Create RPC client for historical queries
    let rpc_client = utils::RpcClient::new(rpc_url).await?;

    let mut upgrades = Vec::new();
    let blocks_queried = to - from + 1;

    // Storage key for System.LastRuntimeUpgrade
    let _last_runtime_upgrade_key =
        "0x26aa394eea5630e07c48ae0c9558cef7f9cce9c888469bb1a0dceaa129672ef8";

    let mut prev_runtime_info: Option<(u32, String)> = None;

    // Query each block
    for block_num in from..=to {
        // Get block hash
        let block_hash: String = rpc_client
            .request("chain_getBlockHash", vec![block_num])
            .await?;

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
    })
}

/// Helper function to fetch detailed information about an upgrade at a specific block
async fn fetch_upgrade_details_at_block(
    upgrade: RuntimeUpgrade,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<RuntimeUpgradeDetails> {
    // Get all events in the upgrade block
    let events_query = EventsQuery {
        from_block: upgrade.block_number as i32,
        to_block: Some(upgrade.block_number as i32),
        pallet: None,
        event: None,
    };

    let events = query_events(events_query, subxt_client, rpc_url).await?;

    // Get all transactions in the upgrade block
    let tx_query = ExtrinsicsQuery {
        from_block: upgrade.block_number as i32,
        to_block: Some(upgrade.block_number as i32),
        pallet: None,
        call: None,
        signer: None,
    };

    let transactions = query_extrinsics(tx_query, subxt_client, rpc_url).await?;

    // Get storage changes (focusing on important system storage)
    let storage_changes =
        get_upgrade_storage_changes(&upgrade.block_hash, upgrade.block_number, rpc_url).await?;

    Ok(RuntimeUpgradeDetails {
        upgrade,
        events: events.events,
        storage_changes,
        transactions: transactions.extrinsics,
    })
}

/// List all runtime changes (upgrades) in a block range with detailed information
pub async fn list_runtime_changes(
    from_block: i32,
    to_block: Option<i32>,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<Vec<RuntimeUpgradeDetails>> {
    // First, find all runtime upgrades in the range
    let query = RuntimeUpgradeQuery {
        from_block,
        to_block,
    };

    let upgrades_result = query_runtime_upgrades(query, subxt_client, rpc_url).await?;

    // If no upgrades found, return empty vec
    if upgrades_result.upgrades.is_empty() {
        return Ok(Vec::new());
    }

    // Process each upgrade sequentially to get detailed information
    let mut details = Vec::new();
    for upgrade in upgrades_result.upgrades {
        match fetch_upgrade_details_at_block(upgrade, subxt_client, rpc_url).await {
            Ok(detail) => details.push(detail),
            Err(e) => {
                // Log warning but continue processing other upgrades
                eprintln!("Warning: Failed to fetch upgrade details: {e}");
            }
        }
    }

    Ok(details)
}

/// Get storage changes for important system keys during upgrade
async fn get_upgrade_storage_changes(
    block_hash: &str,
    block_number: u32,
    rpc_url: &str,
) -> Result<Vec<StorageChange>> {
    // Create RPC client for historical queries
    let rpc_client = utils::RpcClient::new(rpc_url).await?;

    // Get the parent block hash
    let parent_hash: () = rpc_client
        .request("chain_getBlockHash", vec![block_number - 1])
        .await?;

    let mut changes = Vec::new();

    // Important storage keys to check
    let important_keys = vec![
        (
            "0x26aa394eea5630e07c48ae0c9558cef7f9cce9c888469bb1a0dceaa129672ef8",
            "System",
            "LastRuntimeUpgrade",
        ),
        ("0x3a636f6465", "System", ":code"),
        (
            "0x26aa394eea5630e07c48ae0c9558cef79a5f0fe3a994afd1160bf61dd10b857e66",
            "System",
            "UpgradedToU32RefCount",
        ),
        (
            "0x26aa394eea5630e07c48ae0c9558cef7682a096bba730d67f8488aa00d6bcea6",
            "System",
            "UpgradedToTripleRefCount",
        ),
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
