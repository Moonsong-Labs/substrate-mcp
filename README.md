# Basic MCP Server in Rust

This is a minimal stdio-based MCP (Model Context Protocol) server implementation in Rust.

## Features

- Stdio transport (communicates via stdin/stdout)
- Single tool: `hello` - Says hello to someone
- Built with the `rmcp` crate

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Example Tool

The server provides one tool:

- **hello**: Takes a `name` parameter and returns a greeting
  - Input: `{"name": "Alice"}`
  - Output: `"Hello, Alice!"`

## Project Structure

- `src/main.rs` - Main server implementation
- `Cargo.toml` - Dependencies configuration

## Dependencies

- `rmcp` - Rust MCP SDK
- `tokio` - Async runtime
- `serde` & `serde_json` - JSON serialization
- `anyhow` - Error handling

## Next Steps

To extend this server:

1. Add more tools by adding match arms in the `call_tool` method
2. Add resources by implementing the `list_resources` and `read_resource` methods
3. Add prompts by implementing the `list_prompts` and `get_prompt` methods