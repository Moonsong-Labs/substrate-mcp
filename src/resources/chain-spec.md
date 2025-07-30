# Chain Specifications in Substrate

Chain specifications (chain specs) are JSON configuration files that define the initial state and parameters of a blockchain network. They are essential for launching and connecting to Substrate-based chains. This comprehensive guide covers everything you need to know about creating, customizing, and managing chain specifications.

## Overview

A chain specification contains:
- Network identity and properties
- Genesis state configuration
- Boot nodes for peer discovery
- Telemetry endpoints
- Runtime code (Wasm)
- Initial authorities and validators
- Token distribution and balances

## Chain Spec Structure

### Basic Structure

```json
{
  "name": "My Chain",
  "id": "my_chain",
  "chainType": "Local",
  "bootNodes": [],
  "telemetryEndpoints": null,
  "protocolId": "my_protocol",
  "properties": {
    "ss58Format": 42,
    "tokenDecimals": 12,
    "tokenSymbol": "MYT"
  },
  "genesis": {
    "runtime": {
      // Runtime genesis configuration
    }
  }
}
```

### Key Fields Explained

#### Network Identity
```json
{
  "name": "Polkadot",              // Human-readable network name
  "id": "polkadot",                // Unique chain identifier
  "chainType": "Live",             // Chain type: Live, Local, or Development
  "protocolId": "dot",             // Protocol ID for network messages
  "fork_blocks": null,             // Optional: Blocks to fork from
  "bad_blocks": null,              // Optional: Known bad blocks to reject
  "consensusEngine": null,         // Optional: Force specific consensus engine
  "lightSyncState": null           // Optional: Light client sync state
}
```

#### Chain Types
- **Development**: Single validator, instant finality
- **Local**: Local testnet configuration
- **Live**: Production network

#### Properties
```json
"properties": {
  "ss58Format": 0,                 // Address encoding format
  "tokenDecimals": 10,             // Token decimal places
  "tokenSymbol": "DOT",            // Token symbol
  "isEthereum": false              // Optional: Ethereum compatibility
}
```

### Boot Nodes

Boot nodes help new nodes discover peers:

```json
"bootNodes": [
  "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
  "/dns/example.com/tcp/30333/p2p/12D3KooWHKRqRC8Y2Xk3YFsNmPcDNsXKBf7e7PdVphHYjLwXhz7L",
  "/dns/example.com/tcp/30334/ws/p2p/12D3KooWHKRqRC8Y2Xk3YFsNmPcDNsXKBf7e7PdVphHYjLwXhz7L"
]
```

Format: `/[ip4|ip6|dns|dns4|dns6]/<address>/tcp/<port>/p2p/<peer-id>`

### Telemetry

Monitor network health:

```json
"telemetryEndpoints": [
  [
    "wss://telemetry.polkadot.io/submit/",
    0  // Verbosity level (0-9)
  ],
  [
    "wss://telemetry.example.com/submit/",
    1
  ]
]
```

## Genesis Configuration

### Runtime vs Raw Genesis

#### Runtime Genesis (Recommended)
```json
"genesis": {
  "runtime": {
    "system": {
      "code": "0x..." // Runtime Wasm code
    },
    "balances": {
      "balances": [
        ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1000000000000],
        ["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 2000000000000]
      ]
    },
    "sudo": {
      "key": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    },
    "aura": {
      "authorities": [
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
      ]
    },
    "grandpa": {
      "authorities": [
        ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1]
      ]
    }
  }
}
```

#### Raw Genesis (Advanced)
```json
"genesis": {
  "raw": {
    "top": {
      "0x26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac": "0x01000000",
      "0x26aa394eea5630e07c48ae0c9558cef734a5a1d2109e332c7703d063004ac983": "0x00"
    },
    "childrenDefault": {}
  }
}
```

### Common Genesis Pallets

#### System Pallet
```json
"system": {
  "code": "0x...", // Runtime WASM code (required)
  "changesTrieConfig": null // Optional: Changes trie configuration
}
```

#### Balances Pallet
```json
"balances": {
  "balances": [
    ["5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", 1000000000000000],
    ["5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc", 2000000000000000]
  ]
}
```

#### Staking Pallet
```json
"staking": {
  "validatorCount": 4,
  "minimumValidatorCount": 1,
  "invulnerables": [
    "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY"
  ],
  "slashRewardFraction": 100000000,
  "stakers": [
    [
      "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", // Stash
      "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", // Controller
      1000000000000,                                        // Amount
      "Validator"                                           // Status
    ]
  ]
}
```

#### Session Pallet
```json
"session": {
  "keys": [
    [
      "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", // Account
      "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", // Account (again)
      {
        "grandpa": "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
        "babe": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "authority_discovery": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
      }
    ]
  ]
}
```

#### Democracy Pallet
```json
"democracy": {
  "phantom": null // Placeholder for type inference
}
```

#### Council and Technical Committee
```json
"council": {
  "phantom": null,
  "members": [
    "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY",
    "5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc"
  ]
},
"technicalCommittee": {
  "phantom": null,
  "members": [
    "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY"
  ]
}
```

## Generating Chain Specs

### Using the Node Binary

#### 1. Generate Default Spec
```bash
# Generate human-readable chain spec
./target/release/node-template build-spec --disable-default-bootnode > chain-spec.json

# For development
./target/release/node-template build-spec --dev > dev-spec.json

# For local testnet
./target/release/node-template build-spec --local > local-spec.json
```

#### 2. Modify the Spec
Edit the JSON file to customize:
- Initial balances
- Validators
- Sudo key
- Chain properties

#### 3. Convert to Raw Format
```bash
# Convert to raw format (required for production)
./target/release/node-template build-spec \
  --chain=chain-spec.json \
  --raw \
  --disable-default-bootnode > chain-spec-raw.json
```

### Programmatic Generation

```rust
use sc_service::ChainType;
use sp_core::{Pair, Public, sr25519};
use node_template_runtime::{
    AccountId, BalancesConfig, GenesisConfig, SudoConfig,
    SystemConfig, WASM_BINARY, Signature
};

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| 
        "Development wasm not available".to_string()
    )?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some("template"),
        // Fork ID
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    ))
}

fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts.iter().cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        sudo: SudoConfig {
            key: Some(root_key),
        },
        babe: BabeConfig {
            authorities: vec![],
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: GrandpaConfig {
            authorities: vec![],
        },
        // Add other pallets as needed
        ..Default::default()
    }
}
```

## Advanced Topics

### Multi-Chain Networks

For parachains in Polkadot/Kusama:

```json
{
  "name": "My Parachain",
  "id": "my_parachain",
  "chainType": "Live",
  "relay_chain": "polkadot",  // Parent relay chain
  "para_id": 2000,             // Parachain ID
  "properties": {
    "ss58Format": 42,
    "tokenDecimals": 12,
    "tokenSymbol": "MYP"
  },
  "genesis": {
    "runtime": {
      "parachainInfo": {
        "parachainId": 2000
      },
      // Other parachain-specific configuration
    }
  }
}
```

### Custom Types

For chains with custom types:

```json
{
  "name": "Custom Chain",
  "properties": {
    "ss58Format": 42,
    "tokenDecimals": 18,
    "tokenSymbol": "CUST",
    "types": {
      "Address": "MultiAddress",
      "LookupSource": "MultiAddress",
      "MyCustomType": {
        "field1": "u32",
        "field2": "Vec<u8>"
      }
    }
  }
}
```

### Fork Management

Managing chain forks:

```json
{
  "name": "Forked Chain",
  "fork_blocks": [
    [1000, "0x..."], // Fork at block 1000 to block hash
    [2000, "0x..."]  // Fork at block 2000 to block hash
  ],
  "bad_blocks": [
    "0x...", // Known bad block hash to reject
    "0x..."
  ]
}
```

## Working with Chain Specs

### Starting a Node

```bash
# With chain spec file
./node-template --chain=./chain-spec-raw.json

# With built-in spec
./node-template --chain=dev

# As validator
./node-template --chain=./chain-spec-raw.json --validator

# With custom base path
./node-template --chain=./chain-spec-raw.json --base-path /tmp/node
```

### Connecting to Existing Network

```bash
# Connect to network using chain spec
./node-template \
  --chain=./polkadot-spec.json \
  --name="My Node" \
  --pruning=archive

# With bootnodes override
./node-template \
  --chain=./chain-spec.json \
  --bootnodes "/ip4/10.0.0.1/tcp/30333/p2p/12D3KooW..."
```

### Exporting Chain State

```bash
# Export current chain state
./node-template export-state --chain=dev > exported-state.json

# Create new spec from exported state
./node-template build-spec \
  --chain=dev \
  --genesis-state=exported-state.json \
  > new-chain-spec.json
```

## Common Patterns

### Development Chain Spec

```rust
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.unwrap_or_default();
    
    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        move || {
            GenesisConfig {
                system: SystemConfig {
                    code: wasm_binary.to_vec(),
                },
                balances: BalancesConfig {
                    balances: vec![
                        (get_account_id_from_seed::<sr25519::Public>("Alice"), 1_000_000 * DOLLARS),
                        (get_account_id_from_seed::<sr25519::Public>("Bob"), 1_000_000 * DOLLARS),
                    ],
                },
                sudo: SudoConfig {
                    key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
                },
                ..Default::default()
            }
        },
        vec![],
        None,
        Some("dev"),
        None,
        None,
        Default::default(),
    ))
}
```

### Production Chain Spec

```rust
pub fn production_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/chain-spec-raw.json")[..])
}

// Generate with known keys
pub fn production_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
    const STASH: Balance = 100_000 * DOLLARS;
    
    GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY.expect("WASM binary was not built").to_vec(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts.iter()
                .map(|k| (k.clone(), ENDOWMENT))
                .chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
                .collect(),
        },
        session: SessionConfig {
            keys: initial_authorities.iter().map(|x| {
                (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone()))
            }).collect::<Vec<_>>(),
        },
        staking: StakingConfig {
            validator_count: initial_authorities.len() as u32,
            minimum_validator_count: initial_authorities.len() as u32,
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            stakers: initial_authorities.iter().map(|x| {
                (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
            }).collect(),
            ..Default::default()
        },
        ..Default::default()
    }
}
```

## Troubleshooting

### Common Issues

1. **"Bad signature" error**
   - Ensure the correct SS58 format
   - Verify account addresses match the chain's crypto

2. **"Invalid genesis state"**
   - Check that all required pallets are configured
   - Ensure WASM binary is included

3. **"Failed to parse chain spec"**
   - Validate JSON syntax
   - Check for missing required fields

4. **Nodes not connecting**
   - Verify bootnode addresses are correct
   - Check network connectivity
   - Ensure protocol ID matches

### Validation Commands

```bash
# Validate chain spec
./node-template build-spec --chain=chain-spec.json --raw > /dev/null

# Check bootnode address
./node-template key inspect-node-key --file /path/to/node-key

# Generate node key
./node-template key generate-node-key
```

## Best Practices

1. **Version Control**
   - Keep chain specs in version control
   - Tag releases with corresponding runtime versions

2. **Security**
   - Never commit private keys
   - Use hardware security modules for production validators
   - Rotate session keys regularly

3. **Testing**
   - Test chain specs in isolated environments first
   - Verify all genesis parameters
   - Run integration tests with the spec

4. **Documentation**
   - Document any custom types
   - Keep a changelog of spec modifications
   - Include deployment instructions

## External Resources

### Official Documentation
- **Chain Spec Documentation**: https://docs.substrate.io/build/chain-spec/
- **Substrate Node Template**: https://github.com/substrate-developer-hub/substrate-node-template
- **Polkadot Chain Specs**: https://github.com/paritytech/polkadot/tree/master/node/service/res

### Tools
- **Subkey**: Key generation and inspection tool
- **Chain Spec Builder**: https://github.com/paritytech/substrate/tree/master/bin/utils/chain-spec-builder
- **Polkadot JS Apps**: https://polkadot.js.org/apps/#/settings/developer

### Examples
- **Polkadot Spec**: https://github.com/paritytech/polkadot/blob/master/node/service/res/polkadot.json
- **Kusama Spec**: https://github.com/paritytech/polkadot/blob/master/node/service/res/kusama.json
- **Westend Spec**: https://github.com/paritytech/polkadot/blob/master/node/service/res/westend.json

## Important Notes for AI Agents

When working with chain specifications:
1. **Always validate JSON** - Malformed specs will prevent node startup
2. **Check compatibility** - Ensure spec matches the runtime version
3. **Test thoroughly** - Genesis state cannot be changed after launch
4. **Secure keys** - Production keys should never be in plain text
5. **Use raw format** - Production networks require raw chain specs

For specific questions:
- Check the Substrate documentation for detailed explanations
- Look at existing chain specs in production networks
- Test with local dev chains before production deployment
- Consult Substrate StackExchange for common issues

Remember: The chain spec defines the network's initial state and identity. Mistakes in the genesis configuration cannot be fixed without starting a new network.