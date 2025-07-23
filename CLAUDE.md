# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
This is a Rust-based MCP (Model Context Protocol) server designed to provide tools for working with Substrate-based blockchains. The project is in early development (v0.1.0) and currently contains scaffolding for future Substrate-specific functionality.

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

## Architecture

### Core Structure
- **src/main.rs**: Entry point that initializes the MCP server with stdin/stdout transport using tokio async runtime
- **src/server.rs**: Implements `SubstrateService` with MCP tool routing using the `rmcp` crate

### MCP Server Pattern
This server follows the standard MCP communication pattern:
- Uses stdin/stdout for transport (via `rmcp::transport::StdioTransport`)
- Tools are defined using the `#[tool]` macro from rmcp
- Server metadata is defined in the `rmcp::Router` macro

### Current State
The server currently has:
- Basic MCP server infrastructure set up
- One placeholder tool: `say_hello`
- Server info indicating it's for "Tools and Prompts to work with Substrate based blockchains"
- Only tools capability enabled (prompts and resources disabled)

### Adding New Tools
To add new Substrate-related tools:
1. Add tool functions to `src/server.rs` with the `#[tool]` macro
2. Tools should be async functions returning `Result<impl Serialize, anyhow::Error>`
3. Use descriptive names and proper error handling with `anyhow`

Example pattern from existing code:
```rust
#[tool(description = "Tool description")]
async fn tool_name(&self, param: String) -> Result<impl Serialize, anyhow::Error> {
    // Implementation
}
```