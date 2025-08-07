use rmcp::{
    model::{CallToolResult, Content, RawContent, RawTextContent},
    schemars, ErrorData as McpError,
};
use serde::Deserialize;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

use crate::substrate::metadata::{get_call_metadata, CallArgumentInfo};
use crate::substrate::utils::validate_rpc_url;

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct SubmitExtrinsicProperties {
    /// The RPC URL to connect to
    pub rpc_url: String,
    /// The pallet name (e.g., "System", "Balances")
    pub pallet: String,
    /// The call/extrinsic name (e.g., "transfer", "set_code")
    pub call: String,
    /// The arguments for the call as a JSON object that will then be encoded to SCALE.
    /// Use get_call_metadata first to understand the expected format.
    pub args: serde_json::Value,
    /// The dev account to use for signing (alice, bob, charlie, dave, eve, ferdie)
    pub signer: String,
}

pub async fn handle_submit_dev_extrinsic(
    properties: SubmitExtrinsicProperties,
) -> Result<CallToolResult, McpError> {
    // Validate URL
    if let Err(e) = validate_rpc_url(&properties.rpc_url) {
        return Err(McpError {
            code: rmcp::model::ErrorCode(-32602),
            message: format!("Invalid RPC URL: {e}").into(),
            data: None,
        });
    }

    // Connect to the chain using subxt
    let client = OnlineClient::<PolkadotConfig>::from_url(&properties.rpc_url)
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to connect to chain: {e}").into(),
            data: None,
        })?;

    // Validate arguments against metadata
    let metadata = client.metadata();
    let call_metadata = get_call_metadata(&metadata, &properties.pallet, &properties.call)
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32602),
            message: format!("Failed to get call metadata for validation: {e}").into(),
            data: None,
        })?;

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

    // Convert JSON arguments to scale_value::Value using metadata for guidance
    let scale_args = json_to_scale_value_with_metadata(&properties.args, &call_metadata.arguments)
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32602),
            message: format!("Failed to SCALE encode arguments: {e}").into(),
            data: None,
        })?;

    // Create composite from the scale value
    let composite_args = match scale_args.value {
        scale_value::ValueDef::Composite(composite) => composite,
        other => {
            // For non-composite values, wrap in a composite
            let new_value = scale_value::Value {
                value: other,
                context: (),
            };
            scale_value::Composite::from(vec![new_value])
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

    // Wait for the transaction to be finalized
    let tx_in_block = tx_progress
        .wait_for_finalized()
        .await
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to wait for transaction finalization: {e}").into(),
            data: None,
        })?;

    let result = format!(
        "Transaction finalized. Hash: {:?}, Block hash: {:?}",
        tx_in_block.extrinsic_hash(),
        tx_in_block.block_hash()
    );

    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent { text: result }),
        }],
        is_error: None,
    })
}

/// Convert JSON Value to scale_value::Value using metadata for type guidance
fn json_to_scale_value_with_metadata(
    json: &serde_json::Value,
    expected_args: &[CallArgumentInfo],
) -> Result<scale_value::Value, anyhow::Error> {
    match json {
        serde_json::Value::Object(obj) => {
            // Create named composite with proper type guidance
            let mut fields = Vec::new();
            for (key, value) in obj {
                // Find the expected argument info for this field
                let arg_info = expected_args
                    .iter()
                    .find(|arg| arg.name == *key)
                    .ok_or_else(|| anyhow::anyhow!("Unexpected argument: {}", key))?;

                // Convert using type information (we'd need to implement this)
                let converted_value =
                    convert_json_value_with_type_hint(value, &arg_info.type_name)?;
                fields.push((key.as_str(), converted_value));
            }
            Ok(scale_value::Value::named_composite(fields))
        }
        _ => {
            // For single argument calls, use the first expected argument's type
            if expected_args.len() == 1 {
                convert_json_value_with_type_hint(json, &expected_args[0].type_name)
            } else if expected_args.is_empty() && json.is_null() {
                Ok(scale_value::Value::unnamed_composite([]))
            } else {
                anyhow::bail!(
                    "Expected object with {} arguments, got: {:?}",
                    expected_args.len(),
                    json
                );
            }
        }
    }
}

/// Convert a single JSON value using type hint from metadata
fn convert_json_value_with_type_hint(
    json: &serde_json::Value,
    type_hint: &str,
) -> Result<scale_value::Value, anyhow::Error> {
    match type_hint {
        // Primitive types
        "bool" => match json {
            serde_json::Value::Bool(b) => Ok(scale_value::Value::bool(*b)),
            _ => anyhow::bail!("Expected boolean for type 'bool', got: {:?}", json),
        },
        "u8" => convert_to_unsigned_int(json, 8),
        "u16" => convert_to_unsigned_int(json, 16),
        "u32" => convert_to_unsigned_int(json, 32),
        "u64" => convert_to_unsigned_int(json, 64),
        "u128" => convert_to_unsigned_int(json, 128),
        "i8" => convert_to_signed_int(json, 8),
        "i16" => convert_to_signed_int(json, 16),
        "i32" => convert_to_signed_int(json, 32),
        "i64" => convert_to_signed_int(json, 64),
        "i128" => convert_to_signed_int(json, 128),
        "String" | "str" => match json {
            serde_json::Value::String(s) => Ok(scale_value::Value::string(s)),
            _ => anyhow::bail!("Expected string for type '{}', got: {:?}", type_hint, json),
        },

        // Handle Compact types
        t if t.starts_with("Compact<") => {
            // Extract the inner type and convert
            let inner_type = t
                .strip_prefix("Compact<")
                .and_then(|s| s.strip_suffix(">"))
                .ok_or_else(|| anyhow::anyhow!("Invalid Compact type: {}", t))?;
            convert_json_value_with_type_hint(json, inner_type)
        }

        // Handle Vec types
        t if t.starts_with("Vec<") => {
            let inner_type = t
                .strip_prefix("Vec<")
                .and_then(|s| s.strip_suffix(">"))
                .ok_or_else(|| anyhow::anyhow!("Invalid Vec type: {}", t))?;
            match json {
                serde_json::Value::Array(arr) => {
                    let converted: Result<Vec<_>, _> = arr
                        .iter()
                        .map(|item| convert_json_value_with_type_hint(item, inner_type))
                        .collect();
                    Ok(scale_value::Value::unnamed_composite(converted?))
                }
                _ => anyhow::bail!("Expected array for Vec type, got: {:?}", json),
            }
        }

        // Handle AccountId and other common Substrate types
        t if t.contains("AccountId") => {
            match json {
                serde_json::Value::String(s) => {
                    // Assume it's either hex or SS58 format
                    if s.starts_with("0x") {
                        // Hex format - decode to bytes
                        let bytes = hex::decode(s.strip_prefix("0x").unwrap_or(s))
                            .map_err(|e| anyhow::anyhow!("Invalid hex AccountId: {}", e))?;
                        if bytes.len() != 32 {
                            anyhow::bail!("AccountId must be 32 bytes, got {}", bytes.len());
                        }
                        // Convert 32-byte AccountId to proper representation
                        // For now, convert to a fixed array representation
                        let byte_values: Vec<scale_value::Value> = bytes
                            .iter()
                            .map(|b| scale_value::Value::u128(*b as u128))
                            .collect();
                        Ok(scale_value::Value::unnamed_composite(byte_values))
                    } else {
                        // Assume SS58 format - for now just treat as string
                        // TODO: Proper SS58 decoding would require additional dependencies
                        Ok(scale_value::Value::string(s))
                    }
                }
                _ => anyhow::bail!("Expected string for AccountId, got: {:?}", json),
            }
        }

        // Fallback: try basic JSON conversion
        _ => json_to_scale_value_basic(json),
    }
}

/// Convert JSON number to unsigned integer of specific bit width
fn convert_to_unsigned_int(
    json: &serde_json::Value,
    bits: u8,
) -> Result<scale_value::Value, anyhow::Error> {
    match json {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                match bits {
                    8 => Ok(scale_value::Value::u128(u as u8 as u128)),
                    16 => Ok(scale_value::Value::u128(u as u16 as u128)),
                    32 => Ok(scale_value::Value::u128(u as u32 as u128)),
                    64 => Ok(scale_value::Value::u128(u as u128)),
                    128 => Ok(scale_value::Value::u128(u as u128)),
                    _ => anyhow::bail!("Unsupported bit width: {}", bits),
                }
            } else {
                anyhow::bail!("Expected unsigned integer, got: {:?}", n);
            }
        }
        _ => anyhow::bail!("Expected number for u{}, got: {:?}", bits, json),
    }
}

/// Convert JSON number to signed integer of specific bit width  
fn convert_to_signed_int(
    json: &serde_json::Value,
    bits: u8,
) -> Result<scale_value::Value, anyhow::Error> {
    match json {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                match bits {
                    8 => Ok(scale_value::Value::i128(i as i8 as i128)),
                    16 => Ok(scale_value::Value::i128(i as i16 as i128)),
                    32 => Ok(scale_value::Value::i128(i as i32 as i128)),
                    64 => Ok(scale_value::Value::i128(i as i128)),
                    128 => Ok(scale_value::Value::i128(i as i128)),
                    _ => anyhow::bail!("Unsupported bit width: {}", bits),
                }
            } else {
                anyhow::bail!("Expected signed integer, got: {:?}", n);
            }
        }
        _ => anyhow::bail!("Expected number for i{}, got: {:?}", bits, json),
    }
}

/// Basic JSON to scale_value conversion (fallback for unknown types)
fn json_to_scale_value_basic(
    json: &serde_json::Value,
) -> Result<scale_value::Value, anyhow::Error> {
    match json {
        serde_json::Value::Null => Ok(scale_value::Value::unnamed_composite([])),
        serde_json::Value::Bool(b) => Ok(scale_value::Value::bool(*b)),
        serde_json::Value::Number(n) => {
            // Try to preserve the original number type where possible
            if let Some(u) = n.as_u64() {
                Ok(scale_value::Value::u128(u as u128))
            } else if let Some(i) = n.as_i64() {
                Ok(scale_value::Value::i128(i as i128))
            } else {
                anyhow::bail!("Invalid number format: {:?}", n);
            }
        }
        serde_json::Value::String(s) => Ok(scale_value::Value::string(s)),
        serde_json::Value::Array(arr) => {
            let scale_values: Result<Vec<scale_value::Value>, _> =
                arr.iter().map(json_to_scale_value_basic).collect();
            Ok(scale_value::Value::unnamed_composite(scale_values?))
        }
        serde_json::Value::Object(obj) => {
            let mut fields = Vec::new();
            for (key, value) in obj {
                fields.push((key.as_str(), json_to_scale_value_basic(value)?));
            }
            Ok(scale_value::Value::named_composite(fields))
        }
    }
}
