//! Substrate MCP Server Prompt Implementations
//!
//! This module provides individual prompt implementations that are used
//! by the main SubstrateService via delegated function calls.

// Import submodules
pub(crate) mod analyze_release;
pub(crate) mod common;
pub(crate) mod get_started;
pub(crate) mod polkadot_upgrade;
pub(crate) mod release_comparison;
pub(crate) mod scaffold_pallet;
pub(crate) mod security_review;
