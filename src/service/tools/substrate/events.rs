use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

use super::utils;

/// Represents a decoded event from the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DecodedEvent {
    /// The pallet that emitted the event
    pub(crate) pallet: String,
    /// The event variant name
    pub(crate) variant: String,
    /// The block number where the event occurred
    pub(crate) block_number: u32,
    /// The block hash where the event occurred
    pub(crate) block_hash: String,
    /// The event index within the block
    pub(crate) event_index: u32,
    /// The decoded event data as JSON
    pub(crate) data: serde_json::Value,
}

/// Filter criteria for event queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventFilter {
    /// Filter by pallet name (supports partial matching)
    pub(crate) pallet: Option<String>,
    /// Filter by event variant name (supports partial matching)
    pub(crate) variant: Option<String>,
    /// Start block number (inclusive)
    pub(crate) from_block: Option<u32>,
    /// End block number (inclusive)
    pub(crate) to_block: Option<u32>,
    /// Maximum number of events to return
    pub(crate) limit: Option<usize>,
}

/// Query events from blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventsQuery {
    /// Start block (negative = relative to current)
    pub(crate) from_block: i32,
    /// End block (negative = relative to current)
    pub(crate) to_block: Option<i32>,
    /// Optional pallet filter
    pub(crate) pallet: Option<String>,
    /// Optional event name filter
    pub(crate) event: Option<String>,
}

/// Result of events query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventsResult {
    /// Events found
    pub(crate) events: Vec<Event>,
    /// Number of blocks queried
    pub(crate) blocks_queried: u32,
}

/// An event with decoded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Event {
    /// Block number
    pub(crate) block_number: u32,
    /// Block hash
    pub(crate) block_hash: String,
    /// Pallet name
    pub(crate) pallet: String,
    /// Event name
    pub(crate) event: String,
    /// Event index in block
    pub(crate) event_index: u32,
    /// Decoded event data (as JSON)
    pub(crate) data: String,
}

/// Query events using substrate-api-client for RPC and subxt for decoding
pub(crate) async fn query_events(
    query: EventsQuery,
    subxt_client: &OnlineClient<PolkadotConfig>,
    rpc_url: &str,
) -> Result<EventsResult> {
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

    Ok(EventsResult {
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
        if let Some(pallet) = &pallet_filter
            && !event.pallet_name().eq_ignore_ascii_case(pallet)
        {
            continue;
        }

        if let Some(event_name) = &event_filter
            && !event.variant_name().eq_ignore_ascii_case(event_name)
        {
            continue;
        }

        // Decode event data
        let data = match event.field_values() {
            Ok(fields) => format!("{}", fields),
            Err(e) => format!("Failed to decode call arguments: {e}"),
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
