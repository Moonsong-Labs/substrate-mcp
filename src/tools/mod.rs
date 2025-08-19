use rmcp::{
    model::{CallToolResult, Content, RawContent, RawTextContent},
    schemars, ErrorData as McpError,
};
use serde::Deserialize;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

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

pub async fn handle_submit_dev_extrinsic(
    properties: SubmitExtrinsicProperties,
) -> Result<CallToolResult, McpError> {
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to connect to chain: {e}").into(),
            data: None,
        })?;

    let (scale_args_result, remainder) = scale_value::stringify::from_str(&properties.args);
    let scale_args = scale_args_result.map_err(|e| McpError {
        code: rmcp::model::ErrorCode(-32602),
        message: format!("Failed to parse arguments: {e}. See 'substrate:scale-value-format' resource for syntax guide").into(),
        data: None,
    })?;

    if !remainder.trim().is_empty() {
        return Err(McpError {
            code: rmcp::model::ErrorCode(-32602),
            message: format!("Unexpected content after arguments: '{remainder}'").into(),
            data: None,
        });
    }

    // Create composite from the scale value
    // subxt::dynamic::tx expects a Composite, so we need to ensure we have one
    let composite_args = match scale_args.value {
        scale_value::ValueDef::Composite(composite) => composite,
        _ => scale_value::Composite::Unnamed(vec![scale_args]),
    };

    // Get the appropriate signer based on the requested dev account
    let signer = match properties.signer.to_lowercase().as_str() {
        "alice" => dev::alice(),
        "bob" => dev::bob(),
        "charlie" => dev::charlie(),
        "dave" => dev::dave(),
        "eve" => dev::eve(),
        "ferdie" => dev::ferdie(),
        _ => {
            return Err(McpError {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!(
                        "Invalid signer '{}'. Supported signers: alice, bob, charlie, dave, eve, ferdie",
                        properties.signer
                    ).into(),
                    data: None,
                });
        }
    };

    // Create the dynamic call payload
    let call_data = subxt::dynamic::tx(&properties.pallet, &properties.call, composite_args);

    // Submit and wait for finalization
    let tx_progress = client
        .tx()
        .sign_and_submit_then_watch_default(&call_data, &signer)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to submit transaction: {e}").into(),
            data: None,
        })?;

    // Wait for the transaction to be finalized and check for success
    let tx_events = tx_progress
        .wait_for_finalized_success()
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Transaction failed: {e}").into(),
            data: None,
        })?;

    let mut events_info = Vec::new();
    for event in tx_events.iter() {
        let event = event.map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to decode event: {e}").into(),
            data: None,
        })?;

        let fields = event.field_values().map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to decode event fields: {e}").into(),
            data: None,
        })?;

        let value = scale_value::Value {
            value: scale_value::ValueDef::Composite(fields),
            context: 0,
        };
        let fields_str = scale_value::stringify::to_string(&value);

        // Concatenate event name with the fields data
        let event_data = format!(
            "  - {}.{}: {}",
            event.pallet_name(),
            event.variant_name(),
            fields_str
        );

        events_info.push(event_data);
    }

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
