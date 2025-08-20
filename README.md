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

### Event Querying
- **`query_events`** - Query and filter blockchain events within a specified block range. Supports filtering by pallet and event name with partial matching.
- **`query_historical_events`** - Query events from historical blocks. Supports relative block numbers (e.g., -10 for 10 blocks ago).

### Extrinsic Operations
- **`submit_dev_extrinsic`** - Submit an extrinsic to a Substrate chain using dev accounts (alice, bob, charlie, etc.).

### Storage Querying
- **`query_storage`** - Query chain storage entries by pallet and storage name. Supports querying map-type storage with keys.
- **`list_pallet_storage`** - List all storage entries available in a specific pallet.
- **`chain_storage_bisect`** - Find all storage changes between two blocks for a specific key.

### Metadata and Chain Exploration
- **`filter_metadata`** - Filter and search chain metadata to discover available pallets, storage entries, calls, events, constants, and errors. Supports partial name matching.
- **`subxt_execute`** - Use subxt CLI to decode and explore Substrate blockchain data. Useful for analyzing chain metadata, generating type-safe Rust code, and understanding runtime APIs.

### Documentation
- **`fetch_and_analyze_release`** - Fetches and analyzes a Polkadot SDK release by downloading all PRDoc files and generating analysis summaries (manifest, crate changes, audience breakdown). Files are saved to `~/.substrate-mcp/{project}/releases/{release}/pr-docs/` and the tool returns the path for further exploration.

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

## Available Prompts

The Substrate MCP server provides several specialized prompts for Substrate development and security analysis:

### Polkadot SDK Release Analysis

#### release_comparison
**Description**: List changes between two polkadot-sdk release versions  
**Arguments**:
- `current_version` (required): Version currently being used
- `target_version` (required): Version dev wants to compare with (must be greater than current)
- `specific_changes` (optional): What specific changes to look for (e.g: was there any change in `pallet_treasury` ?)

#### analyze_release
**Description**: Analyzes how Polkadot SDK release changes impact your project using parallel processing  
**Arguments**:
- `release` (required): The release version(s) to analyze. Examples: 'stable2503-7' for single release, 'stable2502,stable2503' for comparison
- `focus` (optional): Specific aspect to focus on (e.g., 'breaking changes', 'migrations', 'security'). Leave empty for comprehensive analysis

### Scaffolding

#### scaffold_pallet
**Description**: Generate pallet structure and implementation templates  
**Arguments**:
- `pallet_description` (required): Description for the pallet

### Analysis

#### automated_analysis
**Description**: Template for automated code and runtime analysis  
**Arguments**:
- `change_description` (required): Description of the changes made to the code that trigger this analysis (PR description, new release, etc)

#### code_security_audit
**Description**: Audit specific component for common code-related vulnerabilities  
**Arguments**:
- `audit_target` (required): Describe the target of the audit

#### economic_security
**Description**: Do an economic security analysis on a specific subsystem  
**Arguments**:
- `system_description` (required): Description of the system to make the analysis for (all pallets, a specific group/flow, etc)

#### incentive_analysis
**Description**: Analyze economic viability of incentives  
**Arguments**:
- `target_pallets` (required): List of pallets that make the scope of the analysis
- `analysis_specifications` (required): Specify incentive mechanism to analyze

#### threat_modeling
**Description**: Do threat modeling of a specific part of the system  
**Arguments**:
- `system_description` (required): Description of the system to make the analysis for (all pallets, a specific group/flow, node, etc)

#### weight_analysis
**Description**: Weight-based system breakdown analysis under extreme conditions  
**Arguments**:
- `target_pallet` (required): Pallet to make the analysis for

## License

[LICENSE](LICENSE)
