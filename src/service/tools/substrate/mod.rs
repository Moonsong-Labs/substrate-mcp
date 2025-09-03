/// Client module for Substrate MCP server
/// Metadata module for filtering and querying chain metadata
pub(crate) mod metadata;

/// Events module for querying and filtering chain events
pub(crate) mod events;

/// Storage module for querying chain storage
pub(crate) mod storage;

/// Extrinsic module for querying chain extrinsics
pub(crate) mod extrinsic;

/// Common utility functions used across substrate modules
pub(crate) mod utils;
