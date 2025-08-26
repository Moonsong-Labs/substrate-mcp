use rmcp::ErrorData as McpError;

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
