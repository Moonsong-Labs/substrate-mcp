use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

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
        let query = crate::substrate::historical::HistoricalEventsQuery {
            from_block: from as i32,
            to_block: Some(to as i32),
            pallet: self.pallet.clone(),
            event: self.variant.clone(),
        };
        
        // Get the RPC URL from somewhere (this is a limitation - we need the URL)
        // For now, we'll return an error directing to use the historical events tool
        return Err(anyhow::anyhow!(
            "Historical block hash retrieval not yet implemented. Please use the query_historical_events tool instead."
        ));
    }
    
    /// Check if an event matches the filter criteria
    fn matches_event(&self, pallet: &str, variant: &str) -> bool {
        // Check pallet filter
        if let Some(ref filter_pallet) = self.pallet {
            if !pallet.to_lowercase().contains(&filter_pallet.to_lowercase()) {
                return false;
            }
        }
        
        // Check variant filter
        if let Some(ref filter_variant) = self.variant {
            if !variant.to_lowercase().contains(&filter_variant.to_lowercase()) {
                return false;
            }
        }
        
        true
    }
    
    /// Convert event data to JSON
    fn event_to_json<T>(&self, event: &subxt::events::EventDetails<T>) -> serde_json::Value 
    where
        T: subxt::Config,
    {
        // Get the decoded field values
        match event.field_values() {
            Ok(fields) => {
                // Convert scale_value types to proper JSON
                let json_fields = crate::substrate::scale_utils::composite_to_json(&fields);
                serde_json::json!({
                    "pallet": event.pallet_name(),
                    "variant": event.variant_name(),
                    "fields": json_fields
                })
            }
            Err(e) => {
                // If decoding fails, return error info
                serde_json::json!({
                    "pallet": event.pallet_name(),
                    "variant": event.variant_name(),
                    "error": format!("Failed to decode fields: {}", e)
                })
            }
        }
    }
}

/// Query events from a specific block
pub async fn get_block_events(
    client: &OnlineClient<PolkadotConfig>,
    block_number: Option<u32>,
) -> Result<Vec<DecodedEvent>> {
    // For now, only support querying the latest block
    if block_number.is_some() {
        return Err(anyhow::anyhow!(
            "Historical block query not yet implemented. Please use the query_historical_events tool for historical data."
        ));
    }
    
    let block = client.blocks().at_latest().await?;
    
    let events = block.events().await?;
    let block_number = block.number();
    let block_hash = format!("0x{}", hex::encode(block.hash().as_ref()));
    
    let mut results = Vec::new();
    for (idx, event) in events.iter().enumerate() {
        let event = event?;
        
        let data = match event.field_values() {
            Ok(fields) => {
                let json_fields = crate::substrate::scale_utils::composite_to_json(&fields);
                serde_json::json!({
                    "pallet": event.pallet_name(),
                    "variant": event.variant_name(),
                    "fields": json_fields
                })
            }
            Err(e) => {
                serde_json::json!({
                    "pallet": event.pallet_name(),
                    "variant": event.variant_name(),
                    "error": format!("Failed to decode fields: {}", e)
                })
            }
        };
        
        results.push(DecodedEvent {
            pallet: event.pallet_name().to_string(),
            variant: event.variant_name().to_string(),
            block_number,
            block_hash: block_hash.clone(),
            event_index: idx as u32,
            data,
        });
    }
    
    Ok(results)
}