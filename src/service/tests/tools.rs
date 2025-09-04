use super::helpers::mcp_client::TestMcpClient;
use crate::service::tools::substrate::metadata::MetadataItem;
use crate::service::tools::substrate::storage::StorageResult;
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
