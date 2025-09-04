use super::helpers::mcp_client::TestMcpClient;
use crate::service::tools::substrate::metadata::MetadataItem;
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
