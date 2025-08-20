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
    ├── server.rs       # MCP server implementation with tool routing
    ├── prompts/        # MCP prompt templates
    │   └── mod.rs      # Prompt registry and handlers
    └── substrate/      # Substrate client implementation
        └── client.rs   # RPC client for interacting with nodes

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
- **src/server.rs**: Implements `SubstrateService` with MCP tool routing using the `rmcp` crate

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
- Several functional tools:
  - `fetch_and_analyze_release`: Fetches and analyzes Polkadot SDK releases (downloads PRDocs and generates summaries)
  - `chain_storage_bisect`: Finds storage changes between blocks
- Comprehensive prompt templates for Substrate development:
  - `release_comparison`: Compare changes between Polkadot SDK versions
  - `analyze_release`: Analyze how releases impact your project
  - `scaffold_pallet`: Generate pallet implementation scaffolding
  - `automated_analysis`: Comprehensive security and quality analysis
  - `code_security_audit`: Audit components for vulnerabilities
  - `economic_security`: Economic security assessment
  - `incentive_analysis`: Cryptoeconomic incentive analysis
  - `threat_modeling`: Threat model analysis
  - `weight_analysis`: Weight and benchmark analysis
- Server info indicating it's for Substrate-based blockchain development
- Tools, resources, and prompts capabilities enabled
- Server name: "substrate-mcp" (version 0.1.0)

### Prompt System Architecture

The server uses a modular prompt template system built with Handlebars:

#### Prompt Structure
Each prompt module follows this pattern:
```rust
use rmcp::model::PromptArgument;
use super::SubstratePromptDefinition;

const TEMPLATE: &str = r#"Your prompt template with {{variables}}"#;

pub fn prompt_definition() -> SubstratePromptDefinition {
    SubstratePromptDefinition {
        name: "prompt_name".to_string(),
        description: "What this prompt does".to_string(),
        arguments: vec![
            PromptArgument {
                name: "arg_name".to_string(),
                description: Some("Argument description".to_string()),
                required: Some(true),
            }
        ],
        template: TEMPLATE.to_string(),
    }
}
```

#### Key Components
- **SubstratePromptDefinition**: Core struct containing prompt metadata and template
- **Handlebars Templates**: Support variable interpolation with `{{variable_name}}`
- **Security Disclaimer**: Automatically injected into security-related prompts via `{{security_disclaimer}}`
- **Strict Mode**: Templates use Handlebars strict mode to catch undefined variables

#### Adding New Prompts
1. Create a new module in `src/prompts/` (e.g., `my_prompt.rs`)
2. Define the template constant and prompt_definition function
3. Import the module in `src/prompts/mod.rs`
4. Add to the `prompt_definitions()` vector
5. Write tests following the existing pattern

### Adding New Tools
To add new Substrate-related tools:
1. Add tool functions to `src/server.rs` with the `#[tool]` macro
2. Tools should return `Result<CallToolResult, McpError>`
3. Use descriptive names and proper error handling

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
