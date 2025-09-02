//! Substrate MCP Server Prompt Implementations
//!
//! This module provides individual prompt implementations that are used
//! by the main SubstrateService via delegated function calls.

// Import submodules
pub mod analyze_release;
pub mod automated_analysis;
pub mod code_security_audit;
pub mod common;
pub mod economic_security;
pub mod incentive_analysis;
pub mod release_comparison;
pub mod scaffold_pallet;
pub mod threat_modeling;
pub mod types;
pub mod weight_analysis;
