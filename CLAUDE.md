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

## Architecture

### Core Structure
- **src/main.rs**: Entry point that initializes the MCP server with stdin/stdout transport using tokio async runtime
- **src/server.rs**: Implements `SubstrateService` with MCP tool routing using the `rmcp` crate

### MCP Server Pattern
This server follows the standard MCP communication pattern:
- Uses stdin/stdout for transport (tokio's stdin/stdout)
- Tools are defined using the `#[tool]` macro from rmcp
- Server handler implemented via `#[tool_handler]` macro
- Tool routing via `#[tool_router]` macro

### Current State
The server currently has:
- Basic MCP server infrastructure set up
- Several functional tools:
  - `get_polkadot_sdk_release_prdocs`: Fetches PR documentation for SDK releases
  - `chain_storage_bisect`: Finds storage changes between blocks
- Server info indicating it's for Substrate-based blockchain development
- Both tools and resources capabilities enabled
- Server name: "substrate-mcp" (version 0.1.0)

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
