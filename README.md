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

## Available Prompts

The Substrate MCP server provides several specialized prompts for Substrate development and security analysis:

### release_comparison
**Description**: List changes between two polkadot-sdk release versions  
**Arguments**:
- `current_version` (required): Version currently being used
- `target_version` (required): Version dev wants to compare with (must be greater than current)
- `specific_changes` (optional): What specific changes to look for (e.g: was there any change in `pallet_treasury` ?)

### automated_analysis
**Description**: Template for automated code and runtime analysis  
**Arguments**:
- `change_description` (required): Description of the changes made to the code that trigger this analysis (PR description, new release, etc)

### code_security_audit
**Description**: Audit specific component for common code-related vulnerabilities  
**Arguments**:
- `audit_type` (required): pallet/runtime/node/general
- `audit_target` (required): Describe the target of the audit
- `specific_checks` (optional): Specific things to look for

### economic_security
**Description**: Do an economic security analysis on a specific subsystem  
**Arguments**:
- `system_description` (required): Description of the system to make the analysis for (all pallets, a specific group/flow, etc)
- `extra_context` (required): Extra context to provide for analysis

### pallet_incentive_analysis
**Description**: Analyze economic viability of incentives  
**Arguments**:
- `target_pallets` (required): List of pallets that make the scope of the analysis
- `analysis_specifications` (required): Specific things to look out for during the analysis

### scaffold_pallet
**Description**: Generate pallet structure and implementation templates  
**Arguments**:
- `pallet_description` (required): Description for the pallet

### threat_modeling
**Description**: Do threat modeling of a specific part of the system  
**Arguments**:
- `system_description` (required): Description of the system to make the analysis for (all pallets, a specific group/flow, node, etc)
- `extra_context` (required): Extra context to provide for analysis

### weight_analysis
**Description**: Weight-based system breakdown analysis under extreme conditions  
**Arguments**:
- `target_pallet` (required): Pallet to make the analysis for

## License

[LICENSE](LICENSE)
