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

## Configuration

### RPC Endpoints

The server uses a configuration file (`rpc_endpoints.toml`) to manage RPC endpoints for different Substrate-based networks. This file is optional - if not present, the server will use built-in defaults.

To customize endpoints, create an `rpc_endpoints.toml` file in the same directory where you run the MCP server:

```toml
[[endpoints]]
name = "local"
url = "http://127.0.0.1:9944"
description = "Local development node"

[[endpoints]]
name = "polkadot"
url = "wss://rpc.polkadot.io"
description = "Polkadot mainnet"

[[endpoints]]
name = "my-custom-node"
url = "ws://my-node.example.com:9944"
description = "My custom Substrate node"
```

You can then use endpoint names instead of URLs in tools:
- Use `"polkadot"` instead of `"wss://rpc.polkadot.io"`
- Use `"local"` instead of `"http://127.0.0.1:9944"`
- Direct URLs are still supported for endpoints not in the config

Use the `list_rpc_endpoints` tool to see all available configured endpoints.

## License

[LICENSE](LICENSE)
