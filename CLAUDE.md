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
    └── server.rs       # MCP server implementation with tool routing

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
- One placeholder tool: `say_hello` that returns "Hello, World! From substrate MCP"
- Server info indicating it's for "Tools and Prompts to work with Substrate based blockchains"
- Only tools capability enabled (prompts and resources disabled)
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