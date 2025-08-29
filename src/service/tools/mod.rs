use futures::FutureExt;
use rmcp::{
    model::{CallToolResult, Content, RawContent, RawTextContent},
    schemars, ErrorData as McpError,
};
use serde::Deserialize;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

pub mod polkadot_sdk_releases;
pub mod substrate;
pub mod utils;

use utils::{mcp_error_internal, mcp_error_invalid_params};

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

    let events_info = crate::substrate::utils::get_event_details_from_extrinsic(&tx_events)?;

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
