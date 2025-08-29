use futures::FutureExt;
use rmcp::{
    model::{CallToolResult, Content, RawContent, RawTextContent},
    schemars, ErrorData as McpError,
};
use serde::Deserialize;
use std::process::Stdio;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;
use tokio::process::Command;

pub mod polkadot_sdk_releases;
pub mod substrate;

use self::substrate::events::{query_events, EventsQuery};
use self::substrate::extrinsic::{query_extrinsics, ExtrinsicsQuery};
use self::substrate::metadata::MetadataFilter;
use self::substrate::runtime::list_runtime_changes;
use self::substrate::storage::{list_pallet_storage, query_storage, StorageQuery};
use super::utils::{mcp_error_internal, mcp_error_invalid_params};

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct FetchAndAnalyzeReleaseProperties {
    /// polkadot-sdk release (examples: '1.9.0', 'stable2412-1', 'stable2412')
    pub release: String,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct SubmitExtrinsicProperties {
    /// The RPC URL to connect to
    pub rpc_url: String,
    /// The pallet name (e.g., "System", "Balances")
    pub pallet: String,
    /// The call/extrinsic name (e.g., "transfer", "set_code")
    pub call: String,
    /// The arguments for the call in scale_value string format (see 'substrate:scale-value-format' resource)
    pub args: String,
    /// The dev account to use for signing (alice, bob, charlie, dave, eve, ferdie)
    pub signer: String,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct SubxtExecuteArgs {
    /// The subxt command and arguments to execute (e.g., ["metadata", "-f", "json", "--url", "ws://localhost:9944"])
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct MetadataFilterArgs {
    /// The RPC URL to connect to
    pub rpc_url: String,
    /// Filter by item type (e.g., "pallet", "storage", "call", "event", "constant", "error")
    pub item_type: Option<String>,
    /// Filter by pallet name (supports partial matching)
    pub pallet: Option<String>,
    /// Filter by item name (supports partial matching)
    pub name: Option<String>,
    /// Include detailed type information
    pub include_details: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryEventsProperties {
    /// The RPC endpoint to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
    /// Filter by pallet name (optional)
    pub pallet: Option<String>,
    /// Filter by event name (optional)
    pub event: Option<String>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryStorageProperties {
    /// The RPC URL to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
    /// The pallet name
    pub pallet: String,
    /// The storage entry name
    pub entry: String,
    /// Optional keys for map-type storage (as JSON array). Supports SS58 addresses which will be automatically decoded to AccountId32
    pub keys: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct ListPalletStorageArgs {
    /// The RPC URL to connect to
    pub rpc_url: String,
    /// The pallet name
    pub pallet: String,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryExtrinsicsProperties {
    /// The RPC endpoint to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
    /// Filter by pallet name (optional)
    pub pallet: Option<String>,
    /// Filter by call name (optional)
    pub call: Option<String>,
    /// Filter by signer address (optional)
    pub signer: Option<String>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct ListRuntimeChangesProperties {
    /// The RPC endpoint to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to current block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
}

pub async fn handle_fetch_and_analyze_release(
    properties: FetchAndAnalyzeReleaseProperties,
) -> Result<CallToolResult, McpError> {
    let response = polkadot_sdk_releases::fetch_and_analyze_release(&properties.release)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: e.to_string().into(),
            data: None,
        })?;

    // Format the response as JSON string
    let response_text = serde_json::to_string_pretty(&response).map_err(|e| McpError {
        code: rmcp::model::ErrorCode(-32603),
        message: format!("Failed to serialize response: {e}").into(),
        data: None,
    })?;

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent {
                text: response_text,
            }),
        }],
        is_error: None,
    })
}

pub async fn handle_submit_dev_extrinsic(
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

    let result = format!(
        "Transaction successful!\nHash: {:?}\nEvents emitted:\n{}",
        tx_events.extrinsic_hash(),
        events_info.join("\n")
    );

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: result }),
        }],
        is_error: None,
    })
}

pub async fn handle_subxt_execute(
    args: SubxtExecuteArgs,
) -> Result<CallToolResult, McpError> {
    if args.args.is_empty() {
        return Err(McpError {
            code: rmcp::model::ErrorCode(-32602),
            message: "No arguments provided for subxt command".into(),
            data: None,
        });
    }

    log::info!("Executing subxt with args: {:?}", args.args);

    let output = Command::new("subxt")
        .args(&args.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to execute subxt: {e}. Make sure subxt is installed.")
                .into(),
            data: None,
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("subxt command failed: {stderr}").into(),
            data: None,
        });
    }

    let result = if !stdout.is_empty() {
        stdout.to_string()
    } else if !stderr.is_empty() {
        // Some subxt commands output to stderr even on success
        stderr.to_string()
    } else {
        "Command completed successfully with no output".to_string()
    };

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: result }),
        }],
        is_error: None,
    })
}

pub async fn handle_filter_metadata(
    args: MetadataFilterArgs,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to connect to chain: {e}").into(),
            data: None,
        })?;

    // Get metadata
    let metadata = client.metadata();

    // Create filter
    let filter = MetadataFilter {
        item_type: args.item_type,
        pallet: args.pallet,
        name: args.name,
        include_details: args.include_details.unwrap_or(false),
    };

    // Apply filter
    let results = filter.apply(&metadata).map_err(|e| McpError {
        code: rmcp::model::ErrorCode(-32603),
        message: format!("Failed to filter metadata: {e}").into(),
        data: None,
    })?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&results).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Serialization error: {e}").into(),
        data: None,
    })?;

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: json_result }),
        }],
        is_error: None,
    })
}

pub async fn handle_query_events(
    args: QueryEventsProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to connect to chain: {e}").into(),
            data: None,
        })?;

    // Create query
    let query = EventsQuery {
        from_block: args.from_block,
        to_block: args.to_block,
        pallet: args.pallet,
        event: args.event,
    };

    // Query historical events
    let result = query_events(query, &client, &args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to query historical events: {e}").into(),
            data: None,
        })?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Serialization error: {e}").into(),
        data: None,
    })?;

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: json_result }),
        }],
        is_error: None,
    })
}

pub async fn handle_query_storage(
    args: QueryStorageProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to connect to chain: {e}").into(),
            data: None,
        })?;

    // Create query
    let query = StorageQuery {
        from_block: args.from_block,
        to_block: args.to_block,
        pallet: args.pallet,
        entry: args.entry,
        keys: args.keys,
    };

    // Query historical storage
    let result = query_storage(query, &client, &args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to query historical storage: {e}").into(),
            data: None,
        })?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Serialization error: {e}").into(),
        data: None,
    })?;

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: json_result }),
        }],
        is_error: None,
    })
}

pub async fn handle_list_pallet_storage(
    args: ListPalletStorageArgs,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to connect to chain: {e}").into(),
            data: None,
        })?;

    // List storage entries
    let entries = list_pallet_storage(&client, &args.pallet)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to list storage: {e}").into(),
            data: None,
        })?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&entries).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Serialization error: {e}").into(),
        data: None,
    })?;

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: json_result }),
        }],
        is_error: None,
    })
}

pub async fn handle_query_extrinsics(
    args: QueryExtrinsicsProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!(
                "Failed to connect to chain with URL '{}': {e}",
                args.rpc_url
            )
            .into(),
            data: None,
        })?;

    // Create query
    let query = ExtrinsicsQuery {
        from_block: args.from_block,
        to_block: args.to_block,
        pallet: args.pallet,
        call: args.call,
        signer: args.signer,
    };

    // Query extrinsics
    let result = query_extrinsics(query, &client, &args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to query historical transactions: {e}").into(),
            data: None,
        })?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Serialization error: {e}").into(),
        data: None,
    })?;

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: json_result }),
        }],
        is_error: None,
    })
}

pub async fn handle_list_runtime_changes(
    args: ListRuntimeChangesProperties,
) -> Result<CallToolResult, McpError> {
    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!(
                "Failed to connect to chain with URL '{}': {e}",
                args.rpc_url
            )
            .into(),
            data: None,
        })?;

    // List runtime changes
    let result = list_runtime_changes(args.from_block, args.to_block, &client, &args.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to list runtime changes: {e}").into(),
            data: None,
        })?;

    // Convert to JSON
    let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Serialization error: {e}").into(),
        data: None,
    })?;

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: json_result }),
        }],
        is_error: None,
    })
}
