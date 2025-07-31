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
        let mut results = Vec::new();
        
        // Determine block range
        let latest_block = client.blocks().at_latest().await?;
        let latest_number = latest_block.number();
        
        let from = self.from_block.unwrap_or(latest_number.saturating_sub(100));
        let to = self.to_block.unwrap_or(latest_number);
        
        // Iterate through blocks
        for block_num in from..=to {
            // Get block hash
            // Get block at height
            let _block = client
                .blocks()
                .at_latest()
                .await?;
            
            // Get the hash for the specific block number
            let block_hash = {
                // For now, we'll use a workaround to get historical blocks
                // In a real implementation, you'd use the RPC client directly
                let latest = client.blocks().at_latest().await?;
                if block_num == latest.number() {
                    latest.hash()
                } else {
                    // This is a limitation - we need to implement proper block hash retrieval
                    return Err(anyhow::anyhow!("Historical block hash retrieval not yet implemented"));
                }
            };
            
            // Get block
            let block = client.blocks().at(block_hash).await?;
            
            // Get events
            let events = block.events().await?;
            
            // Process events
            for (idx, event) in events.iter().enumerate() {
                let event = event?;
                
                // Check if event matches filter
                if self.matches_event(event.pallet_name(), event.variant_name()) {
                    // Convert event data to JSON
                    let data = self.event_to_json(&event);
                    
                    results.push(DecodedEvent {
                        pallet: event.pallet_name().to_string(),
                        variant: event.variant_name().to_string(),
                        block_number: block_num,
                        block_hash: format!("0x{}", hex::encode(block_hash.as_ref())),
                        event_index: idx as u32,
                        data,
                    });
                    
                    // Check limit
                    if let Some(limit) = self.limit {
                        if results.len() >= limit {
                            return Ok(results);
                        }
                    }
                }
            }
        }
        
        Ok(results)
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
                // For now, just return the debug representation
                // A full implementation would properly convert scale_value types
                serde_json::json!({
                    "pallet": event.pallet_name(),
                    "variant": event.variant_name(),
                    "fields": format!("{:?}", fields)
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
    let block = if let Some(num) = block_number {
        // For specific block number, we need to get the hash
        // This is a simplified approach - in production, use proper RPC methods
        let latest = client.blocks().at_latest().await?;
        let hash = if num == latest.number() {
            latest.hash()
        } else {
            // For historical blocks, we'd need to use RPC methods
            return Err(anyhow::anyhow!("Historical block query not yet implemented for block {}", num));
        };
        client.blocks().at(hash).await?
    } else {
        client.blocks().at_latest().await?
    };
    
    let events = block.events().await?;
    let block_number = block.number();
    let block_hash = format!("0x{}", hex::encode(block.hash().as_ref()));
    
    let mut results = Vec::new();
    for (idx, event) in events.iter().enumerate() {
        let event = event?;
        
        let data = match event.field_values() {
            Ok(fields) => {
                serde_json::json!({
                    "pallet": event.pallet_name(),
                    "variant": event.variant_name(),
                    "fields": format!("{:?}", fields)
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