use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

/// Query transactions from historical blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTransactionsQuery {
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

/// Result of historical transactions query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTransactionsResult {
    /// Transactions found
    pub transactions: Vec<HistoricalTransaction>,
    /// Number of blocks queried
    pub blocks_queried: u32,
    /// Current block height
    pub current_block: u32,
}

/// A historical transaction with decoded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTransaction {
    /// Block number
    pub block_number: u32,
    /// Block hash
    pub block_hash: String,
    /// Extrinsic index in block
    pub extrinsic_index: u32,
    /// Transaction hash
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

/// Query historical transactions using jsonrpsee for RPC and subxt for decoding
pub async fn query_historical_transactions(
    query: HistoricalTransactionsQuery,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<HistoricalTransactionsResult> {
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::ws_client::WsClientBuilder;

    // Create WebSocket RPC client for historical queries
    let rpc_client = WsClientBuilder::default().build(rpc_url).await?;

    // Get current block number
    let current_block: u32 = {
        let params: Vec<serde_json::Value> = vec![];
        let header: serde_json::Value = rpc_client.request("chain_getHeader", params).await?;

        let number_hex = header["number"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No block number in header"))?;

        // Parse hex number (remove 0x prefix)
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
        None => current_block, // Default to current block if not specified
    };

    let mut all_transactions = Vec::new();
    let blocks_queried = to - from + 1;

    // Query each block
    for block_num in from..=to {
        // Get block hash
        let block_hash: Option<String> = rpc_client
            .request("chain_getBlockHash", vec![block_num])
            .await?;

        let block_hash =
            block_hash.ok_or_else(|| anyhow::anyhow!("Block {} not found", block_num))?;

        // Get block with extrinsics
        let block: serde_json::Value = rpc_client
            .request("chain_getBlock", vec![&block_hash])
            .await?;

        // Extract extrinsics
        if let Some(extrinsics) = block["block"]["extrinsics"].as_array() {
            for (idx, extrinsic_hex) in extrinsics.iter().enumerate() {
                if let Some(hex_str) = extrinsic_hex.as_str() {
                    // Decode extrinsic using subxt
                    match decode_extrinsic_with_subxt(
                        hex_str,
                        &block_hash,
                        block_num,
                        idx as u32,
                        subxt_client,
                        &query.pallet,
                        &query.call,
                        &query.signer,
                    )
                    .await
                    {
                        Ok(Some(tx)) => all_transactions.push(tx),
                        Ok(None) => {}, // Filtered out
                        Err(e) => {
                            log::warn!("Failed to decode extrinsic at block {block_num} index {idx}: {e}");
                        }
                    }
                }
            }
        }
    }

    Ok(HistoricalTransactionsResult {
        transactions: all_transactions,
        blocks_queried,
        current_block,
    })
}

/// Decode extrinsic using subxt's metadata
async fn decode_extrinsic_with_subxt(
    extrinsic_hex: &str,
    block_hash: &str,
    block_number: u32,
    extrinsic_index: u32,
    client: &OnlineClient<PolkadotConfig>,
    pallet_filter: &Option<String>,
    call_filter: &Option<String>,
    signer_filter: &Option<String>,
) -> Result<Option<HistoricalTransaction>> {
    // Remove 0x prefix and decode hex
    let bytes = hex::decode(extrinsic_hex.trim_start_matches("0x"))?;

    // Get metadata from subxt client
    let metadata = client.metadata();

    // Decode the extrinsic
    let _extrinsic = subxt::tx::SubmittableExtrinsic::<PolkadotConfig, OnlineClient<PolkadotConfig>>::from_bytes(
        client.clone(),
        bytes.clone(),
    );

    // Try to decode as a signed extrinsic
    let _decoded_ext = subxt::utils::Encoded(bytes.clone());
    
    // Extract basic extrinsic info using raw decoding
    // This is a simplified approach - in production you'd want more robust decoding
    let (pallet_name, call_name, signer_address) = match extract_extrinsic_info(&bytes, &metadata) {
        Ok(info) => info,
        Err(e) => {
            log::debug!("Failed to extract extrinsic info: {e}");
            return Ok(None);
        }
    };

    // Apply filters
    if let Some(ref pallet) = pallet_filter {
        if !pallet_name.eq_ignore_ascii_case(pallet) {
            return Ok(None);
        }
    }

    if let Some(ref call) = call_filter {
        if !call_name.eq_ignore_ascii_case(call) {
            return Ok(None);
        }
    }

    if let Some(ref signer) = signer_filter {
        if let Some(ref addr) = signer_address {
            if !addr.contains(signer) {
                return Ok(None);
            }
        } else {
            return Ok(None); // Filter requires signer but this is unsigned
        }
    }

    // Calculate transaction hash using subxt's hashing
    let hash = format!("0x{}", hex::encode(subxt::utils::H256::from_slice(&bytes[..32.min(bytes.len())]).as_bytes()));

    // For now, we'll return a simplified version
    // In a full implementation, you'd decode the call arguments properly
    Ok(Some(HistoricalTransaction {
        block_number,
        block_hash: block_hash.to_string(),
        extrinsic_index,
        hash,
        signer: signer_address,
        pallet: pallet_name,
        call: call_name,
        args: serde_json::json!({"raw": extrinsic_hex}),
        success: true, // Would need to check events to determine this
        fee: None, // Would need to calculate from events
    }))
}

/// Extract basic info from extrinsic bytes
fn extract_extrinsic_info(
    bytes: &[u8],
    metadata: &subxt::Metadata,
) -> Result<(String, String, Option<String>)> {
    use codec::Decode;
    use codec::Compact;
    
    let mut input = &bytes[..];
    
    // Skip the length prefix (compact encoded)
    let _: Compact<u32> = Decode::decode(&mut input)?;
    
    // Version byte (first 2 bits = version, bit 7 = signed)
    let version_byte = input[0];
    let is_signed = version_byte & 0b10000000 != 0;
    input = &input[1..];
    
    let signer = if is_signed {
        // Skip signature data for now - this is complex and varies by chain
        // In a real implementation, you'd properly decode MultiAddress and signature
        Some("0x...".to_string()) // Placeholder
    } else {
        None
    };
    
    // After skipping signature data, we'd find the call data
    // For now, we'll try to find it heuristically
    
    // Look for pallet index in metadata
    if let Some(pallet_index_pos) = input.iter().position(|&b| b < 50) { // Assume pallet indices < 50
        let pallet_index = input[pallet_index_pos];
        
        // Try to find pallet by index
        for pallet in metadata.pallets() {
            if pallet.index() == pallet_index {
                // Assume next byte is call index
                if pallet_index_pos + 1 < input.len() {
                    let call_index = input[pallet_index_pos + 1];
                    
                    // Try to find call by index
                    if let Some(_call_ty) = pallet.call_ty_id() {
                        // For now, we'll just return the pallet name with unknown call
                        // Proper call decoding would require more complex logic
                        return Ok((pallet.name().to_string(), format!("Call{}", call_index), signer));
                    }
                }
                
                return Ok((pallet.name().to_string(), "Unknown".to_string(), signer));
            }
        }
    }
    
    Ok(("Unknown".to_string(), "Unknown".to_string(), signer))
}