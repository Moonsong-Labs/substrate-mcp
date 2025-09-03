use futures::FutureExt;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use std::panic::AssertUnwindSafe;

pub fn mcp_error_internal(message: String) -> McpError {
    McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: message.into(),
        data: None,
    }
}

pub fn mcp_error_invalid_params(message: String) -> McpError {
    McpError {
        code: rmcp::model::ErrorCode::INVALID_PARAMS,
        message: message.into(),
        data: None,
    }
}

/// Executes a handler and returns mcp error on panic.
pub async fn catch_panic_as_mcp_error<F>(future: F) -> Result<CallToolResult, McpError>
where
    F: std::future::Future<Output = Result<CallToolResult, McpError>> + Send,
{
    AssertUnwindSafe(future)
        .catch_unwind()
        .await
        .unwrap_or_else(|panic| {
            let panic_msg = if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic".to_string()
            };

            Err(mcp_error_internal(format!(
                "Call panicked: {panic_msg}. This is a bug. Please report it to substrate-mcp mantainers.",
            )))
        })
}
