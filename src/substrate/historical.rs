use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Query events from historical blocks using a hybrid approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEventsQuery {
    /// Start block (negative = relative to current)
    pub from_block: i32,
    /// End block (negative = relative to current)  
    pub to_block: Option<i32>,
    /// Optional pallet filter
    pub pallet: Option<String>,
    /// Optional event name filter
    pub event: Option<String>,
}

/// Result of historical events query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEventsResult {
    /// Events found
    pub events: Vec<HistoricalEvent>,
    /// Number of blocks queried
    pub blocks_queried: u32,
    /// Current block height
    pub current_block: u32,
}

/// A historical event with decoded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    /// Block number
    pub block_number: u32,
    /// Block hash
    pub block_hash: String,
    /// Pallet name
    pub pallet: String,
    /// Event name  
    pub event: String,
    /// Event index in block
    pub event_index: u32,
    /// Decoded event data (as JSON)
    pub data: serde_json::Value,
}

/// Query historical events using substrate-api-client for RPC and subxt for decoding
pub async fn query_historical_events(
    query: HistoricalEventsQuery,
    subxt_client: &subxt::OnlineClient<subxt::PolkadotConfig>,
    rpc_url: &str,
) -> Result<HistoricalEventsResult> {
    use jsonrpsee::ws_client::WsClientBuilder;
    use jsonrpsee::core::client::ClientT;
    
    // Create WebSocket RPC client for historical queries
    let rpc_client = WsClientBuilder::default()
        .build(rpc_url)
        .await?;
    
    // Get current block number
    let current_block: u32 = {
        let params: Vec<serde_json::Value> = vec![];
        let header: serde_json::Value = rpc_client
            .request("chain_getHeader", params)
            .await?;
        
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
    
    let mut all_events = Vec::new();
    let blocks_queried = (to - from + 1) as u32;
    
    // Query each block
    for block_num in from..=to {
        // Get block hash
        let block_hash: Option<String> = rpc_client
            .request("chain_getBlockHash", vec![block_num])
            .await?;
        
        let block_hash = block_hash
            .ok_or_else(|| anyhow::anyhow!("Block {} not found", block_num))?;
        
        // Get storage key for System.Events
        let storage_key = get_events_storage_key();
        
        // Get events at this block
        let events_data: Option<String> = rpc_client
            .request("state_getStorage", (storage_key, &block_hash))
            .await?;
        
        if let Some(events_hex) = events_data {
            // Decode events using subxt metadata
            let events = decode_events_with_subxt(
                &events_hex,
                &block_hash,
                block_num,
                subxt_client,
                &query.pallet,
                &query.event,
            ).await?;
            
            all_events.extend(events);
        }
    }
    
    Ok(HistoricalEventsResult {
        events: all_events,
        blocks_queried,
        current_block,
    })
}

/// Get the storage key for System.Events
fn get_events_storage_key() -> String {
    // System.Events storage key is well-known
    // twox128("System") + twox128("Events")
    "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7".to_string()
}

/// Decode events using subxt's metadata
async fn decode_events_with_subxt(
    events_hex: &str,
    block_hash: &str,
    block_number: u32,
    client: &subxt::OnlineClient<subxt::PolkadotConfig>,
    pallet_filter: &Option<String>,
    event_filter: &Option<String>,
) -> Result<Vec<HistoricalEvent>> {
    use subxt::events::Events;
    
    // Remove 0x prefix and decode hex
    let bytes = hex::decode(&events_hex[2..])?;
    
    // Get metadata from subxt client
    let metadata = client.metadata();
    
    // Decode events using subxt
    let events = Events::<subxt::PolkadotConfig>::decode_from(
        bytes,
        metadata.clone(),
    );
    
    let mut decoded_events = Vec::new();
    
    // Process each event
    for (idx, event) in events.iter().enumerate() {
        let event = event?;
        
        // Apply filters
        if let Some(ref pallet) = pallet_filter {
            if !event.pallet_name().eq_ignore_ascii_case(pallet) {
                continue;
            }
        }
        
        if let Some(ref event_name) = event_filter {
            if !event.variant_name().eq_ignore_ascii_case(event_name) {
                continue;
            }
        }
        
        // Decode event data
        let data = match event.field_values() {
            Ok(fields) => {
                // Convert to proper JSON representation
                let json_fields = crate::substrate::scale_utils::composite_to_json(&fields);
                serde_json::json!({
                    "fields": json_fields,
                    "decoded": true
                })
            }
            Err(e) => {
                serde_json::json!({
                    "error": format!("Failed to decode: {}", e),
                    "decoded": false
                })
            }
        };
        
        decoded_events.push(HistoricalEvent {
            block_number,
            block_hash: block_hash.to_string(),
            pallet: event.pallet_name().to_string(),
            event: event.variant_name().to_string(),
            event_index: idx as u32,
            data,
        });
    }
    
    Ok(decoded_events)
}