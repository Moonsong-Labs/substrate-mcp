use futures::FutureExt;
use rmcp::{
    ErrorData as McpError,
    model::{CallToolResult, Content, RawContent, RawTextContent},
    schemars,
};
use serde::Deserialize;
use std::process::Stdio;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;
use tokio::process::Command;

use super::utils::{mcp_error_internal, mcp_error_invalid_params};

pub(crate) mod polkadot_sdk_releases;
pub(crate) mod runtime_discovery;
pub(crate) mod substrate;

pub(crate) use runtime_discovery::{FindRuntimePalletsProperties, handle_find_runtime_pallets};
use substrate::{
    events::{EventsQuery, query_events},
    extrinsic::{ExtrinsicsQuery, query_extrinsics},
    metadata::MetadataFilter,
    storage::{StorageQuery, list_pallet_storage, query_storage},
};

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub(crate) struct FetchAndAnalyzeReleaseProperties {
    /// polkadot-sdk release (examples: '1.9.0', 'stable2412-1', 'stable2412')
    pub(crate) release: String,
}

pub(crate) async fn handle_fetch_and_analyze_release(
    properties: FetchAndAnalyzeReleaseProperties,
) -> Result<CallToolResult, McpError> {
    let response = polkadot_sdk_releases::fetch_and_analyze_release_enhanced(&properties.release)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to fetch and analyze release: {e}")))?;

    // Format the response as JSON string
    let response_text = serde_json::to_string_pretty(&response)
        .map_err(|e| mcp_error_internal(format!("Failed to serialize response: {e}")))?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: response_text,
            meta: None,
        }),
    }]);
    Ok(result)
}

pub(crate) async fn handle_list_polkadot_releases() -> Result<CallToolResult, McpError> {
    let available_releases = polkadot_sdk_releases::list_available_releases()
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to list available releases: {e}")))?;

    let response_text = serde_json::to_string_pretty(&available_releases)
        .map_err(|e| mcp_error_internal(format!("Failed to serialize response: {e}")))?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: response_text,
        }),
    }]);
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub(crate) struct SubmitExtrinsicProperties {
    /// The RPC URL to connect to
    pub(crate) rpc_url: String,
    /// The pallet name (e.g., "System", "Balances")
    pub(crate) pallet: String,
    /// The call/extrinsic name (e.g., "transfer", "set_code")
    pub(crate) call: String,
    /// The arguments for the call in scale_value string format (see 'substrate:scale-value-format' resource)
    pub(crate) args: String,
    /// The dev account to use for signing (alice, bob, charlie, dave, eve, ferdie)
    pub(crate) signer: String,
}

pub(crate) async fn handle_submit_dev_extrinsic(
    properties: SubmitExtrinsicProperties,
) -> Result<CallToolResult, McpError> {
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to connect to chain: {e}")))?;

    let (scale_args_result, remainder) = scale_value::stringify::from_str(&properties.args);

    let scale_args = scale_args_result.map_err(|e|
        mcp_error_invalid_params(format!("Failed to parse arguments: {e}. See 'substrate:scale-value-format' resource for syntax guide"))
    )?;

    if !remainder.trim().is_empty() {
        return Err(mcp_error_invalid_params(format!(
            "Unexpected content after parsing arguments: '{remainder}'"
        )));
    }

    // Create composite from the scale value
    // subxt::dynamic::tx expects a Composite, so we need to ensure we have one
    let composite_args = match scale_args.value {
        scale_value::ValueDef::Composite(composite) => composite,
        _ => scale_value::Composite::Unnamed(vec![scale_args]),
    };
    let call_data = subxt::dynamic::tx(&properties.pallet, &properties.call, composite_args);

    // Get the appropriate signer based on the requested dev account
    let signer = match properties.signer.to_lowercase().as_str() {
        "alice" => dev::alice(),
        "bob" => dev::bob(),
        "charlie" => dev::charlie(),
        "dave" => dev::dave(),
        "eve" => dev::eve(),
        "ferdie" => dev::ferdie(),
        _ => {
            return Err(mcp_error_invalid_params(format!(
                "Invalid signer '{}'. Supported signers: alice, bob, charlie, dave, eve, ferdie",
                properties.signer
            )));
        }
    };

    // NOTE: `sign_and_submit_then_watch_default` panics when call exists and arguments
    // are valid SCALE but don't fit the call type. We get around this by catching the
    // panic
    let tx_progress = std::panic::AssertUnwindSafe(
        client
            .tx()
            .sign_and_submit_then_watch_default(&call_data, &signer),
    )
    .catch_unwind()
    .await
    .map_err(|_| {
        mcp_error_internal(format!(
            "Transaction submission panicked - likely due to invalid call data. \
            Please verify the call data matches the expectd format for pallet '{}' and call '{}'",
            properties.pallet, properties.call
        ))
    })?
    .map_err(|e| mcp_error_internal(format!("Failed to submit transaction: {e}")))?;

    // Wait for the transaction to be finalized and check for success
    let tx_events = tx_progress
        .wait_for_finalized_success()
        .await
        .map_err(|e| mcp_error_internal(format!("Transaction failed: {e}")))?;

    let events_info = self::substrate::utils::get_event_details_from_extrinsic(&tx_events)?;

    let result_text = format!(
        "Transaction successful!\nHash: {:?}\nEvents emitted:\n{}",
        tx_events.extrinsic_hash(),
        events_info.join("\n")
    );

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: result_text,
            meta: None,
        }),
    }]);
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub(crate) struct SubxtExecuteProperties {
    /// The subxt command and arguments to execute (e.g., ["metadata", "-f", "json", "--url", "ws://localhost:9944"])
    pub(crate) args: Vec<String>,
}

pub(crate) async fn handle_subxt_execute(
    properties: SubxtExecuteProperties,
) -> Result<CallToolResult, McpError> {
    if properties.args.is_empty() {
        return Err(mcp_error_invalid_params(
            "No arguments provided for subxt command".to_string(),
        ));
    }

    log::info!("Executing subxt with args: {:?}", properties.args);

    let output = Command::new("subxt")
        .args(&properties.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            mcp_error_internal(format!(
                "Failed to execute subxt: {e}. Make sure subxt is installed."
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(mcp_error_internal(format!(
            "subxt command failed: {stderr}"
        )));
    }

    let result = if !stdout.is_empty() {
        stdout.to_string()
    } else if !stderr.is_empty() {
        // Some subxt commands output to stderr even on success
        stderr.to_string()
    } else {
        "Command completed successfully with no output".to_string()
    };

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: result,
            meta: None,
        }),
    }]);
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub(crate) struct MetadataFilterProperties {
    /// The RPC URL to connect to
    pub(crate) rpc_url: String,
    /// Filter by item type (e.g., "pallet", "storage", "call", "event", "constant", "error")
    pub(crate) item_type: Option<String>,
    /// Filter by pallet name (supports partial matching)
    pub(crate) pallet: Option<String>,
    /// Filter by item name (supports partial matching)
    pub(crate) name: Option<String>,
    /// Include detailed type information
    pub(crate) include_details: Option<bool>,
}

pub(crate) async fn handle_filter_metadata(
    properties: MetadataFilterProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to connect to chain: {e}")))?;

    // Get metadata
    let metadata = client.metadata();

    // Create filter
    let filter = MetadataFilter {
        item_type: properties.item_type,
        pallet: properties.pallet,
        name: properties.name,
        include_details: properties.include_details.unwrap_or(false),
    };

    // Apply filter
    let results = filter
        .apply(&metadata)
        .map_err(|e| mcp_error_internal(format!("Failed to filter metadata: {e}")))?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&results)
        .map_err(|e| mcp_error_internal(format!("Serialization error: {e}")))?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: json_result,
            meta: None,
        }),
    }]);
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub(crate) struct QueryEventsProperties {
    /// The RPC endpoint to connect to
    pub(crate) rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub(crate) from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub(crate) to_block: Option<i32>,
    /// Filter by pallet name (optional)
    pub(crate) pallet: Option<String>,
    /// Filter by event name (optional)
    pub(crate) event: Option<String>,
}

pub(crate) async fn handle_query_events(
    properties: QueryEventsProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to connect to chain: {e}")))?;

    // Create query
    let query = EventsQuery {
        from_block: properties.from_block,
        to_block: properties.to_block,
        pallet: properties.pallet,
        event: properties.event,
    };

    // Query historical events
    let result = query_events(query, &client, &properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to query events: {e}")))?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&result)
        .map_err(|e| mcp_error_internal(format!("Serialization error: {e}")))?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: json_result,
            meta: None,
        }),
    }]);
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub(crate) struct QueryStorageProperties {
    /// The RPC URL to connect to
    pub(crate) rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub(crate) from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub(crate) to_block: Option<i32>,
    /// The pallet name
    pub(crate) pallet: String,
    /// The storage entry name
    pub(crate) entry: String,
    /// Optional keys for map-type storage (as JSON array). Supports SS58 addresses which will be automatically decoded to AccountId32
    pub(crate) keys: Option<Vec<serde_json::Value>>,
}

pub(crate) async fn handle_query_storage(
    properties: QueryStorageProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to connect to chain: {e}")))?;

    // Create query
    let query = StorageQuery {
        from_block: properties.from_block,
        to_block: properties.to_block,
        pallet: properties.pallet,
        entry: properties.entry,
        keys: properties.keys,
    };

    // Query historical storage
    let result = query_storage(query, &client, &properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to query historical storage: {e}")))?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&result)
        .map_err(|e| mcp_error_internal(format!("Serialization error: {e}")))?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: json_result,
            meta: None,
        }),
    }]);
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub(crate) struct ListPalletStorageProperties {
    /// The RPC URL to connect to
    pub(crate) rpc_url: String,
    /// The pallet name
    pub(crate) pallet: String,
}

pub(crate) async fn handle_list_pallet_storage(
    properties: ListPalletStorageProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to connect to chain: {e}")))?;

    // List storage entries
    let entries = list_pallet_storage(&client, &properties.pallet)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to list storage: {e}")))?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&entries)
        .map_err(|e| mcp_error_internal(format!("Serialization error: {e}")))?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: json_result,
            meta: None,
        }),
    }]);
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub(crate) struct QueryExtrinsicsProperties {
    /// The RPC endpoint to connect to
    pub(crate) rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub(crate) from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub(crate) to_block: Option<i32>,
    /// Filter by pallet name (optional)
    pub(crate) pallet: Option<String>,
    /// Filter by call name (optional)
    pub(crate) call: Option<String>,
    /// Filter by signer address (optional)
    pub(crate) signer: Option<String>,
}

pub(crate) async fn handle_query_extrinsics(
    properties: QueryExtrinsicsProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| {
            mcp_error_internal(format!(
                "Failed to connect to chain with URL '{}': {e}",
                properties.rpc_url
            ))
        })?;

    // Create query
    let query = ExtrinsicsQuery {
        from_block: properties.from_block,
        to_block: properties.to_block,
        pallet: properties.pallet,
        call: properties.call,
        signer: properties.signer,
    };

    // Query extrinsics
    let result = query_extrinsics(query, &client, &properties.rpc_url)
        .await
        .map_err(|e| mcp_error_internal(format!("Failed to query historical transactions: {e}")))?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&result)
        .map_err(|e| mcp_error_internal(format!("Serialization error: {e}")))?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: json_result,
            meta: None,
        }),
    }]);
    Ok(result)
}
