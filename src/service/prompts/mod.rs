//! Substrate MCP Server Prompt Implementations
//!
//! This module provides individual prompt implementations that are used
//! by the main SubstrateService via delegated function calls.

// Import submodules
pub(crate) mod analyze_release;
pub(crate) mod automated_analysis;
pub(crate) mod code_security_audit;
pub(crate) mod common;
pub(crate) mod economic_security;
pub(crate) mod incentive_analysis;
pub(crate) mod release_comparison;
pub(crate) mod scaffold_pallet;
pub(crate) mod threat_modeling;
pub(crate) mod weight_analysis;
