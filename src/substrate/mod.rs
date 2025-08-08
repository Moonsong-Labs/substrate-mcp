/// Client module for Substrate MCP server
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

/// Transactions module for querying chain transactions
pub mod transactions;

/// Runtime upgrades module for tracking and analyzing runtime upgrades
pub mod runtime;

/// Common utility functions used across substrate modules
pub mod utils;
