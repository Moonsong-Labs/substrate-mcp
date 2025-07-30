//! Embedded resource definitions for Substrate development documentation and tools.
//! These resources serve as an index for agents to know where to find official documentation,
//! tools, and references for Substrate blockchain development.

use rmcp::model::{Annotations, RawResource, Resource};

/// Resource URI constants
pub const RESOURCE_SUBSTRATE_DOCS: &str = "substrate:substrate-docs";
pub const RESOURCE_POLKADOT_SDK: &str = "substrate:polkadot-sdk";
pub const RESOURCE_SUBSTRATE_TEMPLATES: &str = "substrate:node-templates";
pub const RESOURCE_SUBSTRATE_TUTORIALS: &str = "substrate:tutorials";
pub const RESOURCE_XCM_DOCS: &str = "substrate:xcm-docs";
pub const RESOURCE_CHAIN_SPEC: &str = "substrate:chain-spec";
pub const RESOURCE_RUST_DOCS: &str = "substrate:rust-docs";
pub const RESOURCE_POLKADOT_JS: &str = "substrate:polkadot-js";
pub const RESOURCE_SUBXT: &str = "substrate:subxt";
pub const RESOURCE_SUBSTRATE_FRONTEND: &str = "substrate:frontend-template";
pub const RESOURCE_INK_DOCS: &str = "substrate:ink-docs";
pub const RESOURCE_POLKADOT_WIKI: &str = "substrate:polkadot-wiki";
pub const RESOURCE_FRAME_BENCHMARKING: &str = "substrate:benchmarking";
pub const RESOURCE_SUBSTRATE_TELEMETRY: &str = "substrate:telemetry";
pub const RESOURCE_ZOMBIENET: &str = "substrate:zombienet";

/// Resource content definitions
pub const SUBSTRATE_DOCS_CONTENT: &str = r#"# Substrate Documentation Hub

Documentation: https://docs.polkadot.com/
API Reference: https://paritytech.github.io/polkadot-sdk/master/sc_service/
Guide: https://paritytech.github.io/polkadot-sdk/book/

Core documentation sections:
- Learn: Concepts, architecture, and theory
- Build: Practical guides for development
- Reference: API documentation and glossary
- Tutorials: Step-by-step learning paths
"#;

pub const POLKADOT_SDK_CONTENT: &str = r#"# Polkadot SDK Reference

Documentation: https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/
GitHub repository: https://github.com/paritytech/polkadot-sdk

The Polkadot SDK (formerly Substrate) includes:
- Substrate framework for custom blockchains
- Polkadot relay chain implementation
- Cumulus for parachain development
- XCM for cross-chain messaging
- Comprehensive Rust documentation
"#;

pub const SUBSTRATE_TEMPLATES_CONTENT: &str = r#"# Substrate Node & Parachain Templates

Node Template: https://github.com/paritytech/polkadot-sdk/tree/master/templates/solochain
Parachain Template: https://github.com/paritytech/polkadot-sdk/tree/master/templates/parachain

Official templates for starting new projects:
- substrate-node-template: Standalone blockchain scaffold
- substrate-parachain-template: Parachain development with Cumulus
- Minimal runtime configuration
- Example pallets and configurations
- Production-ready project structure
- Built-in benchmarking setup
"#;

pub const SUBSTRATE_TUTORIALS_CONTENT: &str = r#"# Substrate Tutorials

Main Tutorial Hub: https://docs.polkadot.com/tutorials/
Learn Substrate: https://docs.polkadot.com/tutorials/build-a-blockchain/

Step-by-step learning paths:
- Build your first Substrate blockchain
- Add pallets to your runtime
- Configure genesis state
- Upgrade a running network
- Create custom pallets
- Implement runtime APIs
- Testing strategies
"#;

pub const XCM_DOCS_CONTENT: &str = r#"# XCM (Cross-Consensus Messaging)

Documentation: https://paritytech.github.io/xcm-docs/
Format specification: https://github.com/paritytech/xcm-format

XCM resources:
- Conceptual overview and design principles
- Message format and instruction reference
- Integration guides for parachains
- Testing and debugging XCM programs
- Common patterns and best practices
"#;

pub const CHAIN_SPEC_CONTENT: &str = r#"# Chain Specifications

Documentation: https://docs.polkadot.com/develop/parachains/deployment/generate-chain-specs/
Reference implementation: https://github.com/paritytech/polkadot-sdk/tree/master/substrate/bin/utils/chain-spec-builder

Chain specification resources:
- Chain spec file format and structure
- Genesis configuration
- Runtime upgrades and migrations
- Network bootstrapping
- Custom chain parameters
"#;

pub const RUST_DOCS_CONTENT: &str = r#"# Rust Documentation for Substrate

Substrate crates.io docs: https://docs.rs/sc-service/latest/
Polkadot SDK rustdocs: https://paritytech.github.io/polkadot-sdk/master/

Key crate documentation:
- sp-runtime: Runtime interfaces and types
- sp-core: Core cryptographic primitives
- frame-support: FRAME macros and utilities
- pallet documentation for all system pallets
"#;

pub const POLKADOT_JS_CONTENT: &str = r#"# Polkadot JS Tools

Apps UI: https://polkadot.js.org/apps/
Documentation: https://polkadot.js.org/docs/
Extension: https://github.com/polkadot-js/extension

JavaScript/TypeScript tools:
- Browser-based blockchain UI
- JavaScript API for Substrate chains
- Browser extension for account management
- Command-line tools and utilities
"#;

pub const SUBXT_CONTENT: &str = r#"# Subxt - Substrate Client Library

GitHub: https://github.com/paritytech/subxt
Documentation: https://docs.rs/subxt/latest/subxt/
Guide: https://docs.rs/subxt/latest/subxt/book/

Rust library for interacting with Substrate nodes:
- Type-safe RPC client
- Dynamic and static metadata support
- Transaction construction and signing
- Event streaming and storage queries
"#;

pub const SUBSTRATE_FRONTEND_CONTENT: &str = r#"# Substrate Frontend Template

GitHub: https://github.com/paritytech/create-polkadot-dapp

Frontend development resources:
- React-based UI template
- Integration with Polkadot JS API
- Account management components
- Transaction submission examples
"#;

pub const INK_DOCS_CONTENT: &str = r#"# ink! Smart Contract Language

Official site: https://use.ink/
Documentation: https://use.ink/docs/
GitHub: https://github.com/paritytech/ink

Smart contracts for Substrate:
- Rust-based eDSL for smart contracts
- Contracts pallet integration
- Development tools and examples
- Testing framework
"#;

pub const POLKADOT_WIKI_CONTENT: &str = r#"# Polkadot Wiki

Official Wiki: https://wiki.polkadot.network/
Learn: https://wiki.polkadot.network/docs/learn-introduction

Comprehensive knowledge base:
- Polkadot concepts and architecture
- Staking and governance guides
- Parachain development
- Ecosystem overview
"#;

pub const BENCHMARKING_CONTENT: &str = r#"# FRAME Benchmarking

Documentation: https://docs.polkadot.com/develop/parachains/testing/benchmarking/
Reference: https://paritytech.github.io/polkadot-sdk/master/frame_benchmarking/

Performance testing tools:
- Weight calculation for extrinsics
- Storage benchmarking
- Hardware requirements analysis
- Optimization guides
"#;

pub const SUBSTRATE_TELEMETRY_CONTENT: &str = r#"# Substrate Telemetry

Live Telemetry: https://telemetry.polkadot.io/
Backend: https://github.com/paritytech/substrate-telemetry

Network monitoring:
- Real-time node statistics
- Network topology visualization
- Performance metrics
- Node version tracking
"#;

pub const ZOMBIENET_CONTENT: &str = r#"# Zombienet - Network Testing Framework

GitHub: https://github.com/paritytech/zombienet
Documentation: https://paritytech.github.io/zombienet/

Testing framework for Substrate networks:
- Spawn ephemeral test networks
- Declarative network configuration
- Multi-parachain testing support
- Integration test automation
- Performance and stress testing
- CI/CD integration
"#;

/// Get all available resources
pub fn get_all_resources() -> Vec<Resource> {
    vec![
        // Core Documentation
        create_resource(
            RESOURCE_SUBSTRATE_DOCS,
            "Substrate Documentation Hub",
            "Main documentation portal for Substrate blockchain framework",
            SUBSTRATE_DOCS_CONTENT,
            0.95,
        ),
        create_resource(
            RESOURCE_POLKADOT_SDK,
            "Polkadot SDK Reference",
            "GitHub repository and documentation for the Polkadot SDK",
            POLKADOT_SDK_CONTENT,
            0.9,
        ),
        // Templates and Tutorials
        create_resource(
            RESOURCE_SUBSTRATE_TEMPLATES,
            "Substrate Node & Parachain Templates",
            "Official templates for starting new blockchain projects",
            SUBSTRATE_TEMPLATES_CONTENT,
            0.85,
        ),
        create_resource(
            RESOURCE_SUBSTRATE_TUTORIALS,
            "Substrate Tutorials",
            "Step-by-step guides for learning Substrate development",
            SUBSTRATE_TUTORIALS_CONTENT,
            0.85,
        ),
        // Development Tools
        create_resource(
            RESOURCE_RUST_DOCS,
            "Rust Documentation for Substrate",
            "API documentation for Substrate crates and modules",
            RUST_DOCS_CONTENT,
            0.8,
        ),
        create_resource(
            RESOURCE_POLKADOT_JS,
            "Polkadot JS Tools",
            "JavaScript/TypeScript tools for Substrate development",
            POLKADOT_JS_CONTENT,
            0.8,
        ),
        create_resource(
            RESOURCE_SUBXT,
            "Subxt Client Library",
            "Rust library for interacting with Substrate nodes",
            SUBXT_CONTENT,
            0.8,
        ),
        create_resource(
            RESOURCE_SUBSTRATE_FRONTEND,
            "Frontend Template",
            "React-based frontend template for Substrate dApps",
            SUBSTRATE_FRONTEND_CONTENT,
            0.8,
        ),
        // Specialized Topics
        create_resource(
            RESOURCE_XCM_DOCS,
            "XCM Documentation",
            "Cross-Consensus Messaging documentation and format specification",
            XCM_DOCS_CONTENT,
            0.7,
        ),
        create_resource(
            RESOURCE_CHAIN_SPEC,
            "Chain Specifications Guide",
            "Documentation for Substrate chain specifications and configuration",
            CHAIN_SPEC_CONTENT,
            0.7,
        ),
        create_resource(
            RESOURCE_INK_DOCS,
            "ink! Smart Contracts",
            "Smart contract development for Substrate",
            INK_DOCS_CONTENT,
            0.7,
        ),
        create_resource(
            RESOURCE_FRAME_BENCHMARKING,
            "FRAME Benchmarking",
            "Performance testing and weight calculation",
            BENCHMARKING_CONTENT,
            0.7,
        ),
        create_resource(
            RESOURCE_ZOMBIENET,
            "Zombienet Testing Framework",
            "Tool for spawning and testing ephemeral Substrate networks",
            ZOMBIENET_CONTENT,
            0.7,
        ),
        // Community and Learning
        create_resource(
            RESOURCE_POLKADOT_WIKI,
            "Polkadot Wiki",
            "Comprehensive knowledge base for Polkadot ecosystem",
            POLKADOT_WIKI_CONTENT,
            0.5,
        ),
        // Infrastructure
        create_resource(
            RESOURCE_SUBSTRATE_TELEMETRY,
            "Substrate Telemetry",
            "Network monitoring and visualization",
            SUBSTRATE_TELEMETRY_CONTENT,
            0.5,
        ),
    ]
}

/// Get resource content by URI
pub fn get_resource_content(uri: &str) -> Option<&'static str> {
    match uri {
        RESOURCE_SUBSTRATE_DOCS => Some(SUBSTRATE_DOCS_CONTENT),
        RESOURCE_POLKADOT_SDK => Some(POLKADOT_SDK_CONTENT),
        RESOURCE_SUBSTRATE_TEMPLATES => Some(SUBSTRATE_TEMPLATES_CONTENT),
        RESOURCE_SUBSTRATE_TUTORIALS => Some(SUBSTRATE_TUTORIALS_CONTENT),
        RESOURCE_XCM_DOCS => Some(XCM_DOCS_CONTENT),
        RESOURCE_CHAIN_SPEC => Some(CHAIN_SPEC_CONTENT),
        RESOURCE_RUST_DOCS => Some(RUST_DOCS_CONTENT),
        RESOURCE_POLKADOT_JS => Some(POLKADOT_JS_CONTENT),
        RESOURCE_SUBXT => Some(SUBXT_CONTENT),
        RESOURCE_SUBSTRATE_FRONTEND => Some(SUBSTRATE_FRONTEND_CONTENT),
        RESOURCE_INK_DOCS => Some(INK_DOCS_CONTENT),
        RESOURCE_POLKADOT_WIKI => Some(POLKADOT_WIKI_CONTENT),
        RESOURCE_SUBSTRATE_TELEMETRY => Some(SUBSTRATE_TELEMETRY_CONTENT),
        RESOURCE_FRAME_BENCHMARKING => Some(BENCHMARKING_CONTENT),
        RESOURCE_ZOMBIENET => Some(ZOMBIENET_CONTENT),
        _ => None,
    }
}

/// Helper function to create a resource
fn create_resource(
    uri: &str,
    name: &str,
    description: &str,
    content: &str,
    priority: f32,
) -> Resource {
    Resource::new(
        RawResource {
            uri: uri.to_string(),
            name: name.to_string(),
            description: Some(description.to_string()),
            mime_type: Some("text/markdown".to_string()),
            size: Some(content.len() as u32),
        },
        Some(Annotations {
            audience: None,
            priority: Some(priority),
            timestamp: None,
        }),
    )
}
