//! Embedded resource definitions for Substrate development documentation and tools.
//! These resources serve as an index for agents to know where to find official documentation,
//! tools, and references for Substrate blockchain development.

use indoc::indoc;
use rmcp::model::{Annotations, RawResource, Resource};

struct SubstrateResource {
    uri: String,
    name: String,
    description: String,
    content: String,
    priority: f32,
}

impl From<SubstrateResource> for Resource {
    fn from(val: SubstrateResource) -> Self {
        Resource::new(
            RawResource {
                uri: val.uri,
                name: val.name,
                description: Some(val.description),
                mime_type: Some("text/markdown".to_string()),
                size: Some(val.content.len() as u32),
            },
            Some(Annotations {
                audience: None,
                priority: Some(val.priority),
                last_modified: None,
            }),
        )
    }
}

fn resources() -> Vec<SubstrateResource> {
    vec![
        SubstrateResource {
            uri: "substrate:polkadot-docs".to_string(),
            name: "Polkadot Documentation Hub".to_string(),
            description: "Main documentation portal for Polkadot blockchain framework".to_string(),
            content: indoc! {r#"
            # Substrate Documentation Hub

            Documentation: https://docs.polkadot.com/
            API Reference: https://paritytech.github.io/polkadot-sdk/master/sc_service/

            Core documentation sections:
            - Learn: Concepts, architecture, and theory
            - Build: Practical guides for development
            - Reference: API documentation and glossary
            - Tutorials: Step-by-step learning paths
            "#}.to_string(),
            priority: 0.95,
        },
        SubstrateResource {
            uri: "substrate:polkadot-sdk".to_string(),
            name: "Polkadot SDK Reference".to_string(),
            description: "GitHub repository and documentation for the Polkadot SDK".to_string(),
            content: indoc! {r#"
            # Polkadot SDK Reference

            Documentation: https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/
            GitHub repository: https://github.com/paritytech/polkadot-sdk
            Guide: https://paritytech.github.io/polkadot-sdk/book/

            The Polkadot SDK (formerly Substrate) includes:
            - Substrate framework for custom blockchains
            - Polkadot relay chain implementation
            - Cumulus for parachain development
            - XCM for cross-chain messaging
            - Comprehensive Rust documentation
            "#}.to_string(),
            priority: 0.9,
        },
        SubstrateResource {
            uri: "substrate:node-templates".to_string(),
            name: "Substrate Node & Parachain Templates".to_string(),
            description: "Official templates for starting new blockchain projects".to_string(),
            content: indoc! {r#"
            # Substrate Node & Parachain Templates

            Node Template: https://github.com/paritytech/polkadot-sdk/tree/master/templates/solochain
            Parachain Template: https://github.com/paritytech/polkadot-sdk/tree/master/templates/parachain

            Official templates for starting new projects:
            - substrate-node-template: Standalone blockchain scaffold
            - substrate-parachain-template: Parachain development with Cumulus
            - Minimal runtime configuration
            - Example pallets and configurations
            - Production-ready project structure
            - Built-in benchmarking setup
            "#}.to_string(),
            priority: 0.85,
        },
        SubstrateResource {
            uri: "substrate:tutorials".to_string(),
            name: "Substrate Tutorials".to_string(),
            description: "Step-by-step guides for learning Substrate development".to_string(),
            content: indoc! {r#"
            # Substrate Tutorials

            Main Tutorial Hub: https://docs.polkadot.com/tutorials/
            Parachain/local blockchain development: https://docs.polkadot.com/tutorials/polkadot-sdk/parachains/zero-to-hero/

            Step-by-step learning paths:
            - Build your first Substrate blockchain
            - Add pallets to your runtime
            - Configure genesis state
            - Upgrade a running network
            - Create custom pallets
            - Implement runtime APIs
            - Testing strategies
            "#}.to_string(),
            priority: 0.85,
        },
        SubstrateResource {
            uri: "substrate:rust-docs".to_string(),
            name: "Rust Documentation for Substrate".to_string(),
            description: "API documentation for Substrate crates and modules".to_string(),
            content: indoc! {r#"
            # Rust Documentation for Substrate

            Substrate crates.io docs: https://docs.rs/sc-service/latest/
            Polkadot SDK rustdocs: https://paritytech.github.io/polkadot-sdk/master/

            Key crate documentation:
            - sp-runtime: Runtime interfaces and types
            - sp-core: Core cryptographic primitives
            - frame-support: FRAME macros and utilities
            - pallet documentation for all system pallets
            "#}.to_string(),
            priority: 0.8,
        },
        SubstrateResource {
            uri: "substrate:polkadot-js".to_string(),
            name: "Polkadot JS Tools".to_string(),
            description: "JavaScript/TypeScript tools for Substrate development".to_string(),
            content: indoc! {r#"
            # Polkadot JS Tools

            Apps UI: https://polkadot.js.org/apps/
            Documentation: https://polkadot.js.org/docs/
            Extension: https://github.com/polkadot-js/extension

            JavaScript/TypeScript tools:
            - Browser-based blockchain UI
            - JavaScript API for Substrate chains
            - Browser extension for account management
            - Command-line tools and utilities
            "#}.to_string(),
            priority: 0.8,
        },
        SubstrateResource {
            uri: "substrate:subxt".to_string(),
            name: "Subxt Client Library".to_string(),
            description: "Rust library for interacting with Substrate nodes".to_string(),
            content: indoc! {r#"
            # Subxt - Substrate Client Library

            GitHub: https://github.com/paritytech/subxt
            Documentation: https://docs.rs/subxt/latest/subxt/
            Guide: https://docs.rs/subxt/latest/subxt/book/

            Rust library for interacting with Substrate nodes:
            - Type-safe RPC client
            - Dynamic and static metadata support
            - Transaction construction and signing
            - Event streaming and storage queries
            "#}.to_string(),
            priority: 0.8,
        },
        SubstrateResource {
            uri: "substrate:frontend-template".to_string(),
            name: "Frontend Template".to_string(),
            description: "React-based frontend template for Substrate dApps".to_string(),
            content: indoc! {r#"
            # Substrate Frontend Template

            GitHub: https://github.com/paritytech/create-polkadot-dapp

            Frontend development resources:
            - React-based UI template
            - Integration with Polkadot JS API
            - Account management components
            - Transaction submission examples
            "#}.to_string(),
            priority: 0.8,
        },
        SubstrateResource {
            uri: "substrate:xcm-docs".to_string(),
            name: "XCM Documentation".to_string(),
            description: "Cross-Consensus Messaging documentation and format specification".to_string(),
            content: indoc! {r#"
            # XCM (Cross-Consensus Messaging)

            Documentation: https://paritytech.github.io/xcm-docs/
            Format specification: https://github.com/paritytech/xcm-format

            XCM resources:
            - Conceptual overview and design principles
            - Message format and instruction reference
            - Integration guides for parachains
            - Testing and debugging XCM programs
            - Common patterns and best practices
            "#}.to_string(),
            priority: 0.7,
        },
        SubstrateResource {
            uri: "substrate:chain-spec".to_string(),
            name: "Chain Specifications Guide".to_string(),
            description: "Documentation for Substrate chain specifications and configuration".to_string(),
            content: indoc! {r#"
            # Chain Specifications

            Documentation: https://docs.polkadot.com/develop/parachains/deployment/generate-chain-specs/
            Reference implementation: https://github.com/paritytech/polkadot-sdk/tree/master/substrate/bin/utils/chain-spec-builder

            Chain specification resources:
            - Chain spec file format and structure
            - Genesis configuration
            - Runtime upgrades and migrations
            - Network bootstrapping
            - Custom chain parameters
            "#}.to_string(),
            priority: 0.7,
        },
        SubstrateResource {
            uri: "substrate:ink-docs".to_string(),
            name: "ink! Smart Contracts".to_string(),
            description: "Smart contract development for Substrate".to_string(),
            content: indoc! {r#"
            # ink! Smart Contract Language

            Official site: https://use.ink/
            Documentation: https://use.ink/docs/
            GitHub: https://github.com/paritytech/ink

            Smart contracts for Substrate:
            - Rust-based eDSL for smart contracts
            - Contracts pallet integration
            - Development tools and examples
            - Testing framework
            "#}.to_string(),
            priority: 0.7,
        },
        SubstrateResource {
            uri: "substrate:benchmarking".to_string(),
            name: "FRAME Benchmarking".to_string(),
            description: "Performance testing and weight calculation".to_string(),
            content: indoc! {r#"
            # FRAME Benchmarking

            Documentation: https://docs.polkadot.com/develop/parachains/testing/benchmarking/
            Reference: https://paritytech.github.io/polkadot-sdk/master/frame_benchmarking/

            Performance testing tools:
            - Weight calculation for extrinsics
            - Storage benchmarking
            - Hardware requirements analysis
            - Optimization guides
            "#}.to_string(),
            priority: 0.7,
        },
        SubstrateResource {
            uri: "substrate:zombienet".to_string(),
            name: "Zombienet Testing Framework".to_string(),
            description: "Tool for spawning and testing ephemeral Substrate networks".to_string(),
            content: indoc! {r#"
            # Zombienet - Network Testing Framework

            GitHub: https://github.com/paritytech/zombienet
            Documentation: https://paritytech.github.io/zombienet/
            Guide: https://docs.polkadot.com/develop/toolkit/parachains/spawn-chains/zombienet/

            Testing framework for Substrate networks:
            - Spawn ephemeral test networks
            - Declarative network configuration
            - Multi-parachain testing support
            - Integration test automation
            - Performance and stress testing
            - CI/CD integration
            "#}.to_string(),
            priority: 0.7,
        },
        SubstrateResource {
            uri: "substrate:polkadot-wiki".to_string(),
            name: "Polkadot Wiki".to_string(),
            description: "Comprehensive knowledge base for Polkadot ecosystem".to_string(),
            content: indoc! {r#"
            # Polkadot Wiki

            Official Wiki: https://wiki.polkadot.network/
            Learn: https://wiki.polkadot.network/docs/learn-introduction

            Comprehensive knowledge base:
            - Polkadot concepts and architecture
            - Staking and governance guides
            - Parachain development
            - Ecosystem overview
            "#}.to_string(),
            priority: 0.5,
        },
        SubstrateResource {
            uri: "substrate:telemetry".to_string(),
            name: "Substrate Telemetry".to_string(),
            description: "Network monitoring and visualization".to_string(),
            content: indoc! {r#"
            # Substrate Telemetry

            Live Telemetry: https://telemetry.polkadot.io/
            Backend: https://github.com/paritytech/substrate-telemetry

            Network monitoring:
            - Real-time node statistics
            - Network topology visualization
            - Performance metrics
            - Node version tracking
            "#}.to_string(),
            priority: 0.5,
        },
        SubstrateResource {
            uri: "substrate:scale-value-format".to_string(),
            name: "scale_value String Format Guide".to_string(),
            description: "Guide for using scale_value string format when submitting extrinsics to Substrate chains".to_string(),
            content: indoc! {r#"
            # scale_value String Format Guide

            When using the `submit_extrinsic` tool, arguments must be provided in scale_value string format. This format is similar to JSON but with some specific syntax for Substrate types.

            ## Basic Types

            ### Numbers
            - Unsigned integers: `123`, `1000000000000`
            - Signed integers: `-42`, `100`
            - Hexadecimal: `0xFF`

            ### Strings
            - Double quoted: `"hello world"`
            - Escape quotes inside: `"say \"hello\""`

            ### Booleans
            - `true` or `false`

            ### Arrays/Sequences
            - Use parentheses (unnamed composite): `(1, 2, 3)`
            - Mixed types: `(123, "hello", true)`
            - Note: Square bracket syntax `[1, 2, 3]` is NOT supported

            ## Important: Variant/Enum Syntax

            Key rules for variants:
            1. Unit variants (no data) MUST use empty parentheses: `None()`, not `None`
            2. The v-prefix syntax `v"VariantName"` ALWAYS requires parentheses: `v"VariantName"(...)`
            3. For Option types specifically:
               - ✅ Correct: `None()`, `Some(42)`
               - ✅ Also valid: `v"Some"(42)`, `v"None"()`
               - ❌ Wrong: `None`, `v"None"` (missing parentheses)

            ## Complex Types

            ### Named Composites (Objects)
            Use curly braces with unquoted field names:
            ```
            { field1: "value", field2: 123, field3: true }
            ```

            ### Unnamed Composites (Tuples)
            Use parentheses:
            ```
            (123, "hello", true)
            ```

            ### Variants (Enums)
            Two syntaxes are supported:

            1. **Standard syntax** (recommended):
            ```
            None()
            Some(42)
            Error("Not found")
            ```

            2. **Alternative v-prefix syntax**:
            ```
            v"None"()          # Unit variant with v-prefix
            v"Some"(42)
            v"Ok"("hello")
            v"Id"((1, 2, 3, 4))
            ```

            ⚠️ IMPORTANT: The v-prefix syntax ALWAYS requires parentheses, even for unit variants.
            `v"None"` without parentheses will fail - use `v"None"()` or `None()`.

            ### Nested Structures
            ```
            {
              sender: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
              receiver: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
              amount: 1000000000000,
              metadata: {
                memo: "Payment for services",
                timestamp: 1234567890
              }
            }
            ```

            ## Common Substrate Types

            ### AccountId (SS58 addresses)
            AccountId in Substrate is typically a 32-byte array. You need to provide it as:
            - A hex string with 0x prefix: `"0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"`
            - Or decode the SS58 address to bytes and provide as a 32-element tuple

            ### Balance
            Use plain numbers (no quotes):
            ```
            1000000000000
            ```

            ### Option<T>
            Use variant syntax:
            ```
            None()              # Unit variant - parentheses required!
            Some(123)           # Standard syntax
            v"None"()           # Alternative v-prefix syntax
            v"Some"(123)        # v-prefix with value
            Some("value")
            v"Some"("value")    # Alternative syntax
            ```
            ⚠️ CRITICAL: Always include parentheses - `v"None"` without `()` will fail

            ## Real Examples

            ### Balance Transfer
            ```
            {
              dest: Id((142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72)),
              value: 1000000000000
            }
            ```
            Note: The `dest` field is a MultiAddress variant. For AccountId32, use `Id((bytes...))` with the 32-byte array representation of the SS58 address.

            ### System Remark
            ```
            {
              remark: (72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33)
            }
            ```
            Note: The `remark` field expects a `Vec<u8>`, so provide the string as a sequence of byte values.

            ### Complex Call (Escrow Creation)
            ```
            {
              seller: (142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72),
              arbitrator: Some((144, 181, 171, 32, 92, 105, 116, 201, 234, 132, 27, 230, 136, 134, 70, 51, 220, 156, 168, 163, 87, 132, 62, 234, 207, 35, 20, 100, 153, 101, 254, 34)),
              amount: 1000000000000,
              deadline: 50,
              description: (80, 97, 121, 109, 101, 110, 116, 32, 102, 111, 114, 32, 100, 105, 103, 105, 116, 97, 108, 32, 97, 114, 116, 119, 111, 114, 107, 32, 78, 70, 84)
            }
            ```
            Note: AccountId32 fields use 32-byte arrays, Option<AccountId32> uses variant syntax `Some((bytes))` or `None()`, and Vec<u8> uses byte sequences.

            ### Escrow Creation with Option Field
            For a call that expects (seller, arbitrator, amount, deadline, description) where arbitrator is Option<AccountId32>:
            ```
            # Using standard syntax:
            ((144, 181, 171, 32, 92, 105, 116, 201, 234, 132, 27, 230, 136, 134, 70, 51, 220, 156, 168, 163, 87, 132, 62, 234, 207, 35, 20, 100, 153, 101, 254, 34), None(), 500000000000000, 50, (87, 101, 98, 32, 100, 101, 118, 101, 108, 111, 112, 109, 101, 110, 116, 32, 115, 101, 114, 118, 105, 99, 101, 115))

            # Using v-prefix syntax (also valid):
            ((144, 181, 171, 32, 92, 105, 116, 201, 234, 132, 27, 230, 136, 134, 70, 51, 220, 156, 168, 163, 87, 132, 62, 234, 207, 35, 20, 100, 153, 101, 254, 34), v"None"(), 500000000000000, 50, (87, 101, 98, 32, 100, 101, 118, 101, 108, 111, 112, 109, 101, 110, 116, 32, 115, 101, 114, 118, 105, 99, 101, 115))
            ```
            Both `None()` and `v"None"()` work - the key is including the parentheses!

            ## Tips

            1. Use `get_call_metadata` first to understand the expected argument structure
            2. Field names in composites are unquoted
            3. String values must be double-quoted
            4. Numbers are unquoted
            5. The parser will tell you exactly where parsing failed if there's an error
            6. When in doubt, use the standard syntax (without v-prefix) as it works for all variants
            7. The v-prefix syntax is an alternative way to write variants - it always requires parentheses after the variant name
            "#}.to_string(),
            priority: 0.9,
        },
    ]
}

/// Get all available resources
pub(crate) fn get_all_resources() -> Vec<Resource> {
    resources().into_iter().map(|r| r.into()).collect()
}

/// Get resource content by URI
pub(crate) fn get_resource_content(uri: &str) -> Option<String> {
    resources()
        .into_iter()
        .find(|r| r.uri == uri)
        .map(|r| r.content)
}
