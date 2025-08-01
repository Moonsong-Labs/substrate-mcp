/// Client module for Substrate MCP server
///
/// This module contains the client that connects to the Substrate node
/// and related methods
pub mod client;

/// Metadata module for filtering and querying chain metadata
pub mod metadata;

/// Events module for querying and filtering chain events
pub mod events;

/// Storage module for querying chain storage
pub mod storage;

/// Historical module for querying past blocks
pub mod historical;

/// Utilities for converting scale values to JSON
pub mod scale_utils;
