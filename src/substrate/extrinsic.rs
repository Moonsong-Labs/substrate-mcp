use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;
use subxt::blocks::{Block, ExtrinsicDetails};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

use crate::substrate::utils;

/// Query extrinsics from blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicsQuery {
    /// Start block (negative = relative to current)
    pub from_block: i32,
    /// End block (negative = relative to current)  
    pub to_block: Option<i32>,
    /// Optional pallet filter for call
    pub pallet: Option<String>,
    /// Optional call name filter
    pub call: Option<String>,
    /// Optional signer address filter
    pub signer: Option<String>,
}

/// Result of extrinsics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicsResult {
    /// Extrinsics found
    pub extrinsics: Vec<Extrinsic>,
    /// Number of blocks queried
    pub blocks_queried: u32,
}

/// A extrinsic with decoded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extrinsic {
    /// Block number
    pub block_number: u32,
    /// Block hash
    pub block_hash: String,
    /// Extrinsic index in block
    pub extrinsic_index: u32,
    /// Extrinsic hash
    pub hash: String,
    /// Signer address (if signed)
    pub signer: Option<String>,
    /// Pallet name
    pub pallet: String,
    /// Call name  
    pub call: String,
    /// Call arguments (as JSON)
    pub args: serde_json::Value,
    /// Whether the extrinsic was successful
    pub success: bool,
    /// Fee paid (if available)
    pub fee: Option<String>,
}

// Create a configurable set of inherent pallets
const COMMON_INHERENT_PALLETS: &[&str] = &["Timestamp", "ParaInherent", "ParachainSystem"];

/// Query extrinsics using jsonrpsee for RPC and subxt for proper decoding
pub async fn query_extrinsics(
    query: ExtrinsicsQuery,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<ExtrinsicsResult> {
    // Get block range from query parameters
    let (from, to) = utils::get_block_range(query.from_block, query.to_block, subxt_client).await?;

    // Create RPC client for historical queries
    let rpc_client = utils::RpcClient::new(rpc_url).await?;

    let mut all_extrinsics = Vec::new();
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

        // Process extrinsics in the block
        let extrinsics = block.extrinsics().await?;

        for (idx, extrinsic_result) in extrinsics.iter().enumerate() {
            // Handle the Result from iterator
            let extrinsic = match extrinsic_result {
                Ok(ext) => ext,
                Err(e) => {
                    log::warn!("Failed to decode extrinsic at block {block_num} index {idx}: {e}");
                    continue;
                }
            };

            // Decode extrinsic using proper subxt APIs
            match process_extrinsic(
                &extrinsic,
                &block,
                subxt_client,
                idx as u32,
                &query.pallet,
                &query.call,
                &query.signer,
            )
            .await
            {
                Ok(Some(tx)) => all_extrinsics.push(tx),
                Ok(None) => {} // Filtered out
                Err(e) => {
                    log::warn!("Failed to process extrinsic at block {block_num} index {idx}: {e}");
                }
            }
        }
    }

    Ok(ExtrinsicsResult {
        extrinsics: all_extrinsics,
        blocks_queried,
    })
}

/// Process an extrinsic using proper subxt APIs
async fn process_extrinsic(
    extrinsic: &ExtrinsicDetails<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    block: &Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    subxt_client: &OnlineClient<PolkadotConfig>,
    extrinsic_index: u32,
    pallet_filter: &Option<String>,
    call_filter: &Option<String>,
    signer_filter: &Option<String>,
) -> Result<Option<Extrinsic>> {
    // Get metadata from the client
    let metadata = subxt_client.metadata();

    // Get pallet and call indices
    let pallet_index = extrinsic.pallet_index();
    let call_index = extrinsic.variant_index();

    // Resolve pallet name from index
    let pallet = metadata
        .pallet_by_index(pallet_index)
        .ok_or_else(|| anyhow::anyhow!("Pallet with index {} not found", pallet_index))?;
    let pallet_name = pallet.name();

    // Get call name (variant name)
    let call_name = if let Some(call_ty) = pallet.call_ty_id() {
        // Get the call type info
        let call_type = metadata
            .types()
            .resolve(call_ty)
            .ok_or_else(|| anyhow::anyhow!("Call type not found"))?;

        // Get variant by index
        if let scale_info::TypeDef::Variant(variants) = &call_type.type_def {
            variants
                .variants
                .iter()
                .find(|v| v.index == call_index)
                .map(|v| v.name.to_string())
                .unwrap_or_else(|| format!("Call{call_index}"))
        } else {
            format!("Call{call_index}")
        }
    } else {
        format!("Call{call_index}")
    };

    // If pallet is an inherent, exclude it
    if !extrinsic.is_signed() && COMMON_INHERENT_PALLETS.contains(&pallet_name) {
        log::debug!("Filtering inherent: {}.{}", pallet_name, call_name);
        return Ok(None); // Filter out this inherent
    }

    // Apply pallet filter
    if let Some(ref pallet) = pallet_filter {
        if !pallet_name.eq_ignore_ascii_case(pallet) {
            return Ok(None);
        }
    }

    // Apply call filter
    if let Some(ref call) = call_filter {
        if !call_name.eq_ignore_ascii_case(call) {
            return Ok(None);
        }
    }

    // Extract signer address
    let signer_address = if extrinsic.is_signed() {
        extrinsic.address_bytes().map(|bytes| {
            let mut account_bytes = [0u8; 32];
            // The address_bytes() returns raw encoded bytes which includes a version prefix
            // For AccountId32: first byte is version (0x00), next 32 bytes are the actual account
            if bytes.len() == 33 && bytes[0] == 0x00 {
                // Skip the first byte (version indicator) and use the next 32 bytes
                account_bytes.copy_from_slice(&bytes[1..33]);
                sp_core::crypto::AccountId32::new(account_bytes).to_ss58check()
            } else if bytes.len() == 32 {
                // Fallback for cases where we get raw 32 bytes without version prefix
                account_bytes.copy_from_slice(bytes);
                sp_core::crypto::AccountId32::new(account_bytes).to_ss58check()
            } else {
                // For other address types or unexpected formats, return hex
                format!("0x{}", hex::encode(bytes))
            }
        })
    } else {
        None
    };

    // Apply signer filter
    if let Some(ref signer) = signer_filter {
        if let Some(ref addr) = signer_address {
            if !addr.contains(signer) {
                return Ok(None);
            }
        } else {
            return Ok(None); // Filter requires signer but this is unsigned
        }
    }

    // Get extrinsic hash using the bytes
    let extrinsic_bytes = extrinsic.bytes();
    let hash = format!(
        "0x{}",
        hex::encode(sp_core::hashing::blake2_256(extrinsic_bytes))
    );

    // Get block info
    let block_number = block.number();
    let block_hash = format!("0x{}", hex::encode(block.hash()));

    // Decode call arguments
    let args = decode_call_args(extrinsic)?;

    // Check events for success/failure and fees
    let (success, fee) = check_extrinsic_events(block, extrinsic_index).await?;

    Ok(Some(Extrinsic {
        block_number,
        block_hash,
        extrinsic_index,
        hash,
        signer: signer_address,
        pallet: pallet_name.to_string(),
        call: call_name,
        args,
        success,
        fee,
    }))
}

/// Decode call arguments to JSON
fn decode_call_args(
    extrinsic: &ExtrinsicDetails<PolkadotConfig, OnlineClient<PolkadotConfig>>,
) -> Result<serde_json::Value> {
    // Try to decode the fields
    match extrinsic.field_values() {
        Ok(fields) => {
            // Convert scale_value to JSON using existing utility
            Ok(crate::substrate::scale_utils::composite_to_json(&fields))
        }
        Err(e) => {
            // If decoding fails, return the raw hex
            log::debug!("Failed to decode call arguments: {e}");
            let extrinsic_bytes = extrinsic.bytes();
            Ok(serde_json::json!({
                "raw": format!("0x{}", hex::encode(extrinsic_bytes))
            }))
        }
    }
}

/// Check events to determine success/failure and extract fees
async fn check_extrinsic_events(
    block: &Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    extrinsic_index: u32,
) -> Result<(bool, Option<String>)> {
    let events = block.events().await?;

    let mut success = false;
    let mut fee = None;

    // Iterate through events
    for event in events.iter() {
        let event = event?;

        // Check if this event is associated with our extrinsic
        if let subxt::events::Phase::ApplyExtrinsic(ext_idx) = event.phase() {
            if ext_idx != extrinsic_index {
                continue;
            }

            // Check for success/failure
            if event.pallet_name() == "System" {
                match event.variant_name() {
                    "ExtrinsicSuccess" => success = true,
                    "ExtrinsicFailed" => success = false,
                    _ => {}
                }
            }

            // Check for fee payment
            if event.pallet_name() == "TransactionPayment"
                && event.variant_name() == "TransactionFeePaid"
            {
                // Try to extract fee amount
                match event.field_values() {
                    Ok(fields) => {
                        let json = crate::substrate::scale_utils::composite_to_json(&fields);
                        if let Some(actual_fee) = json.get("actual_fee") {
                            fee = Some(actual_fee.to_string());
                        }
                    }
                    Err(e) => {
                        log::debug!("Failed to decode fee event: {e}");
                    }
                }
            }
        }
    }

    Ok((success, fee))
}
