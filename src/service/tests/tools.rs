use super::helpers::mcp_client::TestMcpClient;
use super::helpers::substrate_runner::SubstrateRunner;
use crate::service::tools::substrate::events::EventsResult;
use crate::service::tools::substrate::extrinsic::ExtrinsicsResult;
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
        RawContent::Text(RawTextContent { text, .. }) => text,
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
        RawContent::Text(RawTextContent { text, .. }) => text,
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
        RawContent::Text(RawTextContent { text, .. }) => text,
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
async fn test_submit_dev_extrinsic_and_related_queries() {
    // Check if substrate-node is available
    if !SubstrateRunner::is_available() {
        panic!(
            "substrate-node binary is required to run this test. Please install substrate-node first."
        );
    }

    // Spawn a local substrate node
    let _runner = SubstrateRunner::spawn().expect("Failed to spawn a substrate node");

    // Get the WebSocket URL
    let ws_url = _runner.ws_url();

    // Wait for node to fully initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Create MCP client
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    let submit_args = json!({
        "rpc_url": ws_url,
        "pallet": "Balances",
        "call": "transfer_allow_death",
        "args": r#"{
            dest: Id((144, 181, 171, 32, 92, 105, 116, 201, 234, 132, 27, 230, 136, 134, 70, 51, 220, 156, 168, 163, 87, 132, 62, 234, 207, 35, 20, 100, 153, 101, 254, 34)),
            value: 1000000000000
        }"#,
        "signer": "alice"
    });

    let submit_response = client
        .call_tool("submit_dev_extrinsic", submit_args)
        .await
        .expect("Failed to submit extrinsic");

    let submit_content = &submit_response.content[0];
    let submit_text = match &submit_content.raw {
        RawContent::Text(RawTextContent { text, .. }) => text,
        _ => panic!("Expected text content from submit_dev_extrinsic"),
    };

    assert!(
        submit_text.contains("block_hash")
            || submit_text.contains("success")
            || submit_text.contains("0x"),
        "Extrinsic submission should indicate success: {submit_text}",
    );

    let events_args = json!({
        "rpc_url": ws_url,
        "from_block": -10,
        "to_block": 0,
        "pallet": "Balances",
        "event": "Transfer"
    });

    let events_response = client
        .call_tool("query_events", events_args)
        .await
        .expect("Failed to query events");

    let events_content = &events_response.content[0];
    let events_text = match &events_content.raw {
        RawContent::Text(RawTextContent { text, .. }) => text,
        _ => panic!("Expected text content from query_events"),
    };

    let events_result: EventsResult =
        serde_json::from_str(events_text).expect("Should be able to deserialize events response");

    let transfer_event = &events_result.events[0];
    assert_eq!(transfer_event.pallet, "Balances");
    assert_eq!(transfer_event.event, "Transfer");
    assert!(
        transfer_event.block_number > 0,
        "Event should have block number"
    );
    assert!(
        transfer_event.data.contains("from")
            || transfer_event.data.contains("to")
            || transfer_event.data.contains("amount"),
        "Transfer event should contain transfer data: {}",
        transfer_event.data
    );

    let extrinsics_args = json!({
        "rpc_url": ws_url,
        "from_block": -10,
        "to_block": 0,
        "pallet": "Balances",
        "call": "transfer_allow_death"
    });

    let extrinsics_response = client
        .call_tool("query_extrinsics", extrinsics_args)
        .await
        .expect("Failed to query extrinsics");

    let extrinsics_content = &extrinsics_response.content[0];
    let extrinsics_text = match &extrinsics_content.raw {
        RawContent::Text(RawTextContent { text, .. }) => text,
        _ => panic!("Expected text content from query_extrinsics"),
    };

    // Parse and verify extrinsics response
    let extrinsics_result: ExtrinsicsResult = serde_json::from_str(extrinsics_text)
        .expect("Should be able to deserialize extrinsics response");

    let first_extrinsic = &extrinsics_result.extrinsics[0];
    assert_eq!(first_extrinsic.pallet, "Balances");
    assert_eq!(first_extrinsic.call, "transfer_allow_death");
    assert!(
        first_extrinsic.block_number > 0,
        "Extrinsic should have block number"
    );
    assert!(
        first_extrinsic.signer.is_some(),
        "Extrinsic should have signer information"
    );
    assert!(
        !first_extrinsic.args.is_empty(),
        "Extrinsic should have arguments"
    );

    println!(
        "Found {} extrinsics across {} blocks",
        extrinsics_result.extrinsics.len(),
        extrinsics_result.blocks_queried
    );
}
