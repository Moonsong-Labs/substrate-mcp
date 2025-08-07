/// Validates if a string is a valid RPC URL textually (without connecting)
pub fn validate_rpc_url(url: &str) -> Result<(), String> {
    // Check if it starts with a valid protocol
    if !url.starts_with("ws://")
        && !url.starts_with("wss://")
        && !url.starts_with("http://")
        && !url.starts_with("https://")
    {
        return Err("URL must start with ws://, wss://, http://, or https://".to_string());
    }

    // Basic URL structure validation
    if url.len() < 10 {
        // Minimum: ws://a.b
        return Err("URL is too short".to_string());
    }

    // Check for basic URL structure
    let after_protocol = if let Some(stripped) = url.strip_prefix("ws://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("wss://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("http://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else {
        return Err("Invalid protocol".to_string());
    };

    // Must have at least one character after protocol
    if after_protocol.is_empty() {
        return Err("URL must have a host after the protocol".to_string());
    }

    // Check for spaces or invalid characters
    if url.contains(' ') || url.contains('\n') || url.contains('\t') {
        return Err("URL contains invalid whitespace characters".to_string());
    }

    Ok(())
}
