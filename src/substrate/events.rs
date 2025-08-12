use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

use crate::substrate::utils;

/// Represents a decoded event from the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedEvent {
    /// The pallet that emitted the event
    pub pallet: String,
    /// The event variant name
    pub variant: String,
    /// The block number where the event occurred
    pub block_number: u32,
    /// The block hash where the event occurred
    pub block_hash: String,
    /// The event index within the block
    pub event_index: u32,
    /// The decoded event data as JSON
    pub data: serde_json::Value,
}

/// Filter criteria for event queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Filter by pallet name (supports partial matching)
    pub pallet: Option<String>,
    /// Filter by event variant name (supports partial matching)
    pub variant: Option<String>,
    /// Start block number (inclusive)
    pub from_block: Option<u32>,
    /// End block number (inclusive)
    pub to_block: Option<u32>,
    /// Maximum number of events to return
    pub limit: Option<usize>,
}

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
pub struct HistoricalEventsQueryResult {
    /// Events found
    pub events: Vec<Event>,
    /// Number of blocks queried
    pub blocks_queried: u32,
}

/// A historical event with decoded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
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

impl EventFilter {
    /// Query events from the chain based on the filter criteria
    pub async fn query_events(
        &self,
        client: &OnlineClient<PolkadotConfig>,
    ) -> Result<Vec<DecodedEvent>> {
        // For now, we'll use the historical events module for all event queries
        // This provides a workaround for the block hash retrieval issue

        // Get latest block to determine range
        let latest_block = client.blocks().at_latest().await?;
        let latest_number = latest_block.number();

        let from = self.from_block.unwrap_or(latest_number.saturating_sub(100));
        let to = self.to_block.unwrap_or(latest_number);

        // Create a historical query
        let _query = HistoricalEventsQuery {
            from_block: from as i32,
            to_block: Some(to as i32),
            pallet: self.pallet.clone(),
            event: self.variant.clone(),
        };

        // Get the RPC URL from somewhere (this is a limitation - we need the URL)
        // For now, we'll return an error directing to use the historical events tool
        Err(anyhow::anyhow!(
            "Historical block hash retrieval not yet implemented. Please use the query_historical_events tool instead."
        ))
    }
}

/// Query historical events using substrate-api-client for RPC and subxt for decoding
pub async fn query_historical_events(
    query: HistoricalEventsQuery,
    subxt_client: &subxt::OnlineClient<subxt::PolkadotConfig>,
    rpc_url: &str,
) -> Result<HistoricalEventsQueryResult> {
    // Get block range from query parameters
    let (from, to) = utils::get_block_range(query.from_block, query.to_block, subxt_client).await?;

    // Create WebSocket RPC client for historical queries
    let rpc_client = utils::RpcClient::new(rpc_url).await?;

    let mut all_events = Vec::new();
    let blocks_queried = to - from + 1;

    // Query each block
    for block_num in from..=to {
        // Get block hash
        let block_hash: String = rpc_client
            .request("chain_getBlockHash", vec![block_num])
            .await?;

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
            )
            .await?;

            all_events.extend(events);
        }
    }

    Ok(HistoricalEventsQueryResult {
        events: all_events,
        blocks_queried,
    })
}

/// Decode events using subxt's metadata
async fn decode_events_with_subxt(
    events_hex: &str,
    block_hash: &str,
    block_number: u32,
    client: &subxt::OnlineClient<subxt::PolkadotConfig>,
    pallet_filter: &Option<String>,
    event_filter: &Option<String>,
) -> Result<Vec<Event>> {
    use subxt::events::Events;

    // Remove 0x prefix and decode hex
    let bytes = hex::decode(&events_hex[2..])?;

    // Get metadata from subxt client
    let metadata = client.metadata();

    // Decode events using subxt
    let events = Events::<subxt::PolkadotConfig>::decode_from(bytes, metadata.clone());

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

        decoded_events.push(Event {
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

/// Get the storage key for System.Events
fn get_events_storage_key() -> String {
    // System.Events storage key is well-known
    // twox128("System") + twox128("Events")
    "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7".to_string()
}
