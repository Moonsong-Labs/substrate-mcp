use super::helpers::mcp_client::TestMcpClient;
use crate::service::tools::substrate::metadata::MetadataItem;
use crate::service::tools::substrate::storage::StorageResult;
use crate::service::tools::substrate::events::EventsResult;
use rmcp::model::{RawContent, RawTextContent};
use serde_json::json;

#[tokio::test]
async fn test_tool_filter_metadata() {
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    let args = json!({
        "rpc_url": "wss://rpc.polkadot.io",
        "pallet": "System",
        "item_type": "storage"
    });

    let response = client.call_tool("filter_metadata", args).await.unwrap();

    let content = &response.content[0];
    let text = match &content.raw {
        RawContent::Text(RawTextContent { text }) => text,
        _ => panic!("Expected text content"),
    };
    let metadata_items: Vec<MetadataItem> = serde_json::from_str(text)
        .expect("Should be able to deserialize response as Vec<MetadataItem>");

    for item in &metadata_items {
        assert_eq!(
            item.item_type, "storage",
            "All items should be storage type"
        );
        assert_eq!(
            item.pallet, "System",
            "All items should be from System pallet"
        );
        assert!(item.name.is_some(), "Storage items should have names");
    }

    let item_names: Vec<&str> = metadata_items
        .iter()
        .filter_map(|item| item.name.as_deref())
        .collect();

    assert!(
        item_names.contains(&"Account"),
        "Should contain Account storage item: {item_names:?}"
    );
}

#[tokio::test]
async fn test_tool_list_pallet_storage() {
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    let args = json!({
        "rpc_url": "wss://rpc.polkadot.io",
        "pallet": "System"
    });

    let response = client.call_tool("list_pallet_storage", args).await.unwrap();

    let content = &response.content[0];
    let text = match &content.raw {
        RawContent::Text(RawTextContent { text }) => text,
        _ => panic!("Expected text content"),
    };

    let storage_entries: Vec<String> =
        serde_json::from_str(text).expect("Should be able to deserialize response as Vec<String>");

    assert!(
        storage_entries.contains(&"Account".to_string()),
        "Should contain Account storage entry: {storage_entries:?}"
    );

    assert!(
        storage_entries.contains(&"BlockHash".to_string()),
        "Should contain BlockHash storage entry: {storage_entries:?}"
    );

    assert!(
        storage_entries.contains(&"Number".to_string()),
        "Should contain Number storage entry: {storage_entries:?}"
    );
}

#[tokio::test]
async fn test_tool_query_storage() {
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    let args = json!({
        "rpc_url": "wss://rpc.polkadot.io",
        "from_block": 0,
        "pallet": "System",
        "entry": "LastRuntimeUpgrade"
    });

    let response = client.call_tool("query_storage", args).await.unwrap();

    let content = &response.content[0];
    let text = match &content.raw {
        RawContent::Text(RawTextContent { text }) => text,
        _ => panic!("Expected text content"),
    };

    let storage_result: StorageResult = serde_json::from_str(text)
        .expect("Should be able to deserialize response as StorageResult");

    assert_eq!(storage_result.blocks_queried, 1);
    assert_eq!(storage_result.storage.len(), 1);

    let storage_entry = &storage_result.storage[0];

    assert_eq!(storage_entry.pallet, "System");
    assert_eq!(storage_entry.entry, "LastRuntimeUpgrade");
    assert_eq!(storage_entry.key, "System.LastRuntimeUpgrade");
    assert!(storage_entry.at_block.is_some(), "Should have block number");

    let value_obj = storage_entry.value.as_object().unwrap();

    assert!(
        value_obj.contains_key("decoded"),
        "Existing storage should have decoded field"
    );

    let decoded = value_obj.get("decoded").unwrap().as_str().unwrap();
    assert!(
        decoded.contains("spec_version"),
        "LastRuntimeUpgrade should contain spec_version: {decoded}"
    );
    assert!(
        decoded.contains("spec_name"),
        "LastRuntimeUpgrade should contain spec_name: {decoded}"
    );
    assert!(
        decoded.to_lowercase().contains("polkadot"),
        "LastRuntimeUpgrade should reference polkadot"
    );
}

#[tokio::test]
async fn test_tool_query_events() {
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    // Query System::ExtrinsicSuccess events from last 2 blocks - these occur in every block
    let args = json!({
        "rpc_url": "wss://rpc.polkadot.io",
        "from_block": -2,
        "to_block": -1,
        "pallet": "System",
        "event": "ExtrinsicSuccess"
    });

    let response = client.call_tool("query_events", args).await.unwrap();

    let content = &response.content[0];
    let text = match &content.raw {
        RawContent::Text(RawTextContent { text }) => text,
        _ => panic!("Expected text content"),
    };

    // Deserialize the JSON response into EventsResult
    let events_result: EventsResult = serde_json::from_str(text)
        .expect("Should be able to deserialize response as EventsResult");

    // Validate basic structure
    assert_eq!(events_result.blocks_queried, 2, "Should query exactly 2 blocks");
    assert!(
        !events_result.events.is_empty(),
        "Should return at least one ExtrinsicSuccess event"
    );

    // ExtrinsicSuccess events should occur in every block, so we expect multiple events
    assert!(
        events_result.events.len() >= 2,
        "Should have at least 2 ExtrinsicSuccess events across 2 blocks: found {}",
        events_result.events.len()
    );

    let mut block_numbers = std::collections::HashSet::new();

    // Validate each event
    for event in &events_result.events {
        // Validate event structure
        assert_eq!(event.pallet, "System");
        assert_eq!(event.event, "ExtrinsicSuccess");
        assert!(event.block_number > 0, "Block number should be positive");
        assert!(!event.block_hash.is_empty(), "Block hash should not be empty");
        assert!(
            event.block_hash.starts_with("0x"),
            "Block hash should start with 0x: {}",
            event.block_hash
        );
        assert!(
            event.block_hash.len() == 66,
            "Block hash should be 66 characters (0x + 64 hex): {}",
            event.block_hash
        );

        // Validate event data structure
        assert!(!event.data.is_empty(), "Event data should not be empty");
        
        // Parse the event data to validate structure
        let data_json: serde_json::Value = serde_json::from_str(&event.data)
            .expect("Event data should be valid JSON");
        
        assert!(data_json.is_object(), "Event data should be an object");
        
        // ExtrinsicSuccess should have dispatch_info field
        if let Some(fields) = data_json.get("fields") {
            assert!(
                fields.get("dispatch_info").is_some(),
                "ExtrinsicSuccess should have dispatch_info: {}",
                event.data
            );
        }

        block_numbers.insert(event.block_number);
    }

    // Should have events from 2 different blocks
    assert!(
        block_numbers.len() >= 1,
        "Should have events from at least 1 block"
    );
    
    // Verify block numbers are sequential (recent blocks)
    let sorted_blocks: Vec<u32> = block_numbers.iter().cloned().collect();
    if sorted_blocks.len() >= 2 {
        let max_block = *sorted_blocks.iter().max().unwrap();
        let min_block = *sorted_blocks.iter().min().unwrap();
        assert!(
            max_block - min_block <= 10,
            "Block numbers should be close together (recent blocks): min={}, max={}",
            min_block,
            max_block
        );
    }
}
