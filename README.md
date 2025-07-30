# Substrate MCP Server

An MCP (Model Context Protocol) server that provides tools for working with Substrate-based blockchains.


## Installation

### Option 1: Install from GitHub
```bash
cargo install --git https://github.com/Moonsong-Labs/substrate-mcp
```

### Option 2: Build locally (Requires cargo)
```bash
cargo build --release
```
The binary will be available at `./target/release/substrate-mcp`

## Prerequisites

For the `subxt_execute` tool, install the subxt CLI:
```bash
cargo install subxt-cli
```

## Available Tools

- **`get_polkadot_sdk_release_prdocs`** - Get documented changes for polkadot-sdk releases
- **`chain_storage_bisect`** - Find storage changes between blocks for a specific key
- **`subxt_execute`** - Decode metadata, generate type-safe code, and explore Substrate chains

## Usage with Claude Code

To use this MCP server with Claude Code, add it to your Claude Code configuration.

```json
{
  "mcpServers": {
    "substrate-mcp": {
      "command": "substrate-mcp"
    }
  }
}
```

If you built the server locally instead of installing it, use the full path:

```json
{
  "mcpServers": {
    "substrate-mcp": {
      "command": "/path/to/substrate-mcp/target/release/substrate-mcp"
    }
  }
}
```

Alternatively, you can add using cli:

```claude mcp add substrate /path/to/substrate-mcp/target/release/substrate-mcp```

## License

[LICENSE](LICENSE)
