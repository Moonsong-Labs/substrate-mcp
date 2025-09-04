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

    // Extract the JSON text from the response
    let content = &response.content[0];
    let text = match &content.raw {
        RawContent::Text(RawTextContent { text }) => text,
        _ => panic!("Expected text content"),
    };

    // Deserialize the JSON response into Vec<MetadataItem>
    let metadata_items: Vec<MetadataItem> = serde_json::from_str(text)
        .expect("Should be able to deserialize response as Vec<MetadataItem>");

    // Basic checks on the deserialized data
    assert!(
        !metadata_items.is_empty(),
        "Should return at least one metadata item"
    );

    // Verify all items are storage type and System pallet as requested
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

    // Verify we get some well-known System storage items
    let item_names: Vec<&str> = metadata_items
        .iter()
        .filter_map(|item| item.name.as_deref())
        .collect();

    assert!(
        item_names.contains(&"Account"),
        "Should contain Account storage item: {item_names:?}"
    );
}
