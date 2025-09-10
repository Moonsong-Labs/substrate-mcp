# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
This is a Rust-based MCP (Model Context Protocol) server designed to provide tools for working with Substrate-based blockchains. The project is in early development (v0.1.0) and currently contains scaffolding for future Substrate-specific functionality.

## Repository Structure
```
substrate-mcp/
├── Cargo.toml          # Project manifest with dependencies
├── Cargo.lock          # Dependency lock file
├── LICENSE             # Project license
├── README.md           # User-facing documentation with installation/usage instructions
├── CLAUDE.md           # This file - development guidance
└── src/
    ├── main.rs         # Entry point with tokio async runtime
    └── service/        # MCP service implementation
        ├── mod.rs      # Main service with tool and prompt routing
        ├── prompts/    # MCP prompt templates
        │   ├── mod.rs                      # Prompt module registry
        │   ├── analyze_release.rs          # Release impact analysis
        │   ├── get_started.rs              # Getting started guide
        │   ├── release_comparison.rs       # Release comparison
        │   ├── scaffold_pallet.rs          # Pallet scaffolding
        │   ├── security_review.rs          # Security review (merged from multiple security prompts)
        │   └── common.rs                   # Common prompt utilities
        ├── resources/  # MCP resources (documentation, references)
        ├── tools/      # MCP tools implementation
        └── utils.rs    # Service utilities

## Development Commands

### Build
```bash
cargo build
```

### Run
```bash
cargo run
```

### Test
```bash
cargo test
```

### Lint and Format
```bash
cargo clippy
cargo fmt
```

## Dependencies
- **rmcp** (v0.3.0): MCP server framework with transport-io feature
- **tokio** (v1): Async runtime with full features
- **serde** (v1): Serialization/deserialization with derive feature
- **serde_json** (v1): JSON support
- **anyhow** (v1): Error handling
- **handlebars** (v6): Template engine for prompt generation

## Architecture

### Core Structure
- **src/main.rs**: Entry point that initializes the MCP server with stdin/stdout transport using tokio async runtime
- **src/service/mod.rs**: Implements `SubstrateService` with MCP tool and prompt routing using the `rmcp` crate

### MCP Server Pattern
This server follows the standard MCP communication pattern:
- Uses stdin/stdout for transport (tokio's stdin/stdout)
- Tools are defined using the `#[tool]` macro from rmcp
- Prompts are defined as `SubstratePromptDefinition` structs with Handlebars templates
- Server handler implemented via `#[tool_handler]` macro
- Tool routing via `#[tool_router]` macro
- Prompt routing via `handle_list_prompts` and `handle_get_prompt` functions

### Current State
The server currently has:
- Basic MCP server infrastructure set up
- Multiple functional tools for blockchain interaction:
  - `fetch_and_analyze_release`: Fetches and analyzes Polkadot SDK releases (downloads PRDocs and generates summaries)
  - `subxt_execute`: Execute subxt commands for blockchain exploration
  - `filter_metadata`: Filter and search chain metadata
  - `query_extrinsics`: Query extrinsics from blocks
  - `query_events`: Query events from blocks
  - `query_storage`: Query chain storage entries
  - `list_pallet_storage`: List storage entries in a pallet
  - `submit_dev_extrinsic`: Submit extrinsics using dev accounts
- Current prompt templates for Substrate development:
  - `release_comparison`: Compare changes between Polkadot SDK versions
  - `analyze_release`: Analyze how releases impact your project
  - `scaffold_pallet`: Generate pallet implementation scaffolding
  - `security_review`: Comprehensive security review covering code security, economic threats, and performance analysis
  - `get_started`: Get started guide for Substrate/Polkadot development
- Server info indicating it's for Substrate-based blockchain development
- Tools, resources, and prompts capabilities enabled
- Server name: "substrate-mcp" (version 0.1.0)

### Prompt System Architecture

The server uses a modular prompt template system built with Handlebars:

#### Prompt Structure
Each prompt module follows this pattern:
```rust
use handlebars::Handlebars;
use rmcp::model::{PromptMessage, PromptMessageRole};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Arguments for the prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Description of what this prompt does")]
pub(crate) struct PromptArgs {
    #[schemars(description = "Description of the argument")]
    pub(crate) arg_name: String,
}

/// Generate prompt content
pub(crate) async fn generate_prompt(args: PromptArgs) -> Vec<PromptMessage> {
    let handlebars = Handlebars::new();
    
    let context = json!({
        "arg_name": args.arg_name,
        "security_disclaimer": SECURITY_DISCLAIMER // if needed
    });

    let content = handlebars
        .render_template(TEMPLATE, &context)
        .unwrap_or_else(|e| format!("Template rendering failed: {}", e));

    vec![PromptMessage::new_text(PromptMessageRole::User, content)]
}

const TEMPLATE: &str = r#"Your prompt template with {{variables}}"#;
```

#### Key Components
- **JsonSchema derive**: Arguments use schemars for automatic schema generation
- **Handlebars Templates**: Support variable interpolation with `{{variable_name}}`
- **Security Disclaimer**: Automatically injected into security-related prompts via `{{security_disclaimer}}`
- **Async Functions**: All prompt generation is async and returns `Vec<PromptMessage>`

#### Adding New Prompts
1. Create a new module in `src/service/prompts/` (e.g., `my_prompt.rs`)
2. Define the args struct with JsonSchema derive and generate_prompt function
3. Import the module in `src/service/prompts/mod.rs`
4. Add prompt handler to `src/service/mod.rs` using `#[prompt]` macro
5. Write tests following the existing pattern

### Adding New Tools
To add new Substrate-related tools:
1. Add tool functions to `src/service/mod.rs` with the `#[tool]` macro
2. Tools should return `Result<CallToolResult, McpError>`
3. Use descriptive names and proper error handling
4. Implementation can be in separate modules in `src/service/tools/`

Example pattern from existing code:
```rust
#[tool(description = "Says hello to the substrate user")]
pub fn say_hello(&self) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult {
        content: vec![Content {
            annotations: None,
            raw: RawContent::Text(RawTextContent {
                text: "Hello, World! From substrate MCP".to_string(),
            }),
        }],
        is_error: None,
    })
}
```

## Installation and Usage
The project can be installed via:
- `cargo install --git https://github.com/Moonsong-Labs/substrate-mcp`
- Local build: `cargo build --release`

For Claude Code configuration, see README.md for detailed setup instructions.

## Substrate Integration Guidelines

### Preferred Crates
When interacting with Substrate nodes, always prefer using the official `polkadot-sdk` crates over third-party alternatives:
- Use `sc-rpc-api` for RPC client traits
- Use `sp-core` for core types and primitives
- Use `sp-runtime` for runtime types
- Use `jsonrpsee` for WebSocket client (this is what polkadot-sdk uses internally)

### RPC Communication Pattern
For RPC communication with Substrate nodes:
1. Use `jsonrpsee` to create WebSocket clients
2. Use the traits from `sc-rpc-api` for type safety
3. Prefer simple sequential async/await over complex parallelization
4. Natural rate limiting through sequential requests is often sufficient

### Code Design Principles
1. **Simplicity First**: Start with simple sequential code before adding complexity
2. **Avoid Over-engineering**: Don't use parallelization libraries like `rayon` for I/O-bound operations
3. **Use Async/Await**: Substrate RPC operations are I/O-bound, use tokio's async runtime
4. **Error Handling**: Use `anyhow` for error propagation in tool implementations

### Common RPC Methods for Storage
When working with storage:
- `state_getKeysPaged`: Fetch storage keys with pagination
- `state_getStorage`: Get storage value at specific block
- `chain_getBlockHash`: Get block hash from block number
- Always handle SCALE encoding/decoding appropriately
