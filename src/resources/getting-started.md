# Getting Started with Substrate

Substrate is a modular framework that enables you to create purpose-built blockchains by composing custom or pre-built components. This guide provides comprehensive information to help you understand and work with Substrate effectively.

## Core Concepts

### 1. Runtime
The runtime is the business logic of your blockchain, defining how transactions are processed and how state transitions occur. It's compiled to WebAssembly (Wasm) for platform independence and forkless upgrades.

**Key characteristics:**
- Written in Rust using the FRAME framework
- Upgradeable without hard forks
- Deterministic execution environment
- Modular architecture through pallets

### 2. Pallets
Pallets are modular components that encapsulate domain-specific logic. Think of them as blockchain "plugins" that can be combined to build your runtime.

**Common patterns:**
- **Storage**: Define on-chain state
- **Events**: Emit notifications about state changes
- **Errors**: Handle failure cases gracefully
- **Extrinsics**: Define callable functions
- **Hooks**: Execute logic at specific block lifecycle points

### 3. Extrinsics
External data submitted to the blockchain that can trigger state changes. They come in two forms:
- **Signed transactions**: Include a signature and pay fees
- **Unsigned transactions**: No signature, used for specific scenarios
- **Inherents**: Data provided by block authors (e.g., timestamp)

### 4. Storage
Substrate provides a powerful storage abstraction with automatic encoding/decoding:
- **Storage Value**: Single values
- **Storage Map**: Key-value mappings
- **Storage Double Map**: Two-key mappings
- **Storage N Map**: N-key mappings

## Development Workflow

### Environment Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# Install additional tools
cargo install --locked cargo-contract # For smart contracts
cargo install --locked cargo-expand   # For macro debugging
```

### Creating a New Substrate Node

```bash
# Clone the node template
git clone https://github.com/substrate-developer-hub/substrate-node-template
cd substrate-node-template

# Build the node
cargo build --release

# Run the node in development mode
./target/release/node-template --dev
```

## Pallet Development Patterns

### Basic Pallet Structure

```rust
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Example: Custom types
        type Currency: Currency<Self::AccountId>;
        
        /// Example: Runtime constants
        #[pallet::constant]
        type MaximumOwned: Get<u32>;
    }
    
    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Something<T> = StorageValue<_, u32>;
    
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when something is stored
        SomethingStored { something: u32, who: T::AccountId },
    }
    
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive
        NoneValue,
        /// Errors should have helpful documentation
        StorageOverflow,
    }
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            // Check the origin
            let who = ensure_signed(origin)?;
            
            // Perform checks
            ensure!(something > 0, Error::<T>::NoneValue);
            
            // Update storage
            <Something<T>>::put(&something);
            
            // Emit an event
            Self::deposit_event(Event::SomethingStored { something, who });
            
            Ok(())
        }
    }
}
```

### Common Implementation Patterns

#### Working with Balances
```rust
use frame_support::traits::{Currency, ReservableCurrency};

// Transfer tokens
T::Currency::transfer(&from, &to, amount, ExistenceRequirement::KeepAlive)?;

// Reserve tokens
T::Currency::reserve(&who, amount)?;

// Unreserve tokens
T::Currency::unreserve(&who, amount);
```

#### Storage Patterns
```rust
// Storage map with complex key
#[pallet::storage]
pub type Kitties<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<KittyId, T::MaxKittiesOwned>,
    ValueQuery,
>;

// Double map for relationships
#[pallet::storage]
pub type KittyPrices<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,  // Owner
    Blake2_128Concat,
    KittyId,       // Kitty ID
    BalanceOf<T>,  // Price
>;
```

## RPC and Client Integration

### Connecting to a Node

```rust
use jsonrpsee::{core::client::ClientT, ws_client::WsClientBuilder};
use sp_core::H256;

// Connect to node
let client = WsClientBuilder::default()
    .build("ws://localhost:9944")
    .await?;

// Make RPC calls
let block_hash: H256 = client
    .request("chain_getBlockHash", rpc_params![None])
    .await?;
```

### Common RPC Methods

```javascript
// Get current block
const blockHash = await api.rpc.chain.getBlockHash();

// Query storage
const balance = await api.query.system.account(address);

// Submit transaction
const transfer = api.tx.balances.transfer(recipient, amount);
const hash = await transfer.signAndSend(sender);

// Subscribe to events
api.query.system.events((events) => {
    events.forEach((record) => {
        console.log(`Event: ${record.event.section}:${record.event.method}`);
    });
});
```

## Testing Strategies

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_ok, assert_noop};
    
    #[test]
    fn it_works_for_default_value() {
        new_test_ext().execute_with(|| {
            // Dispatch a signed extrinsic
            assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
            // Check storage
            assert_eq!(TemplateModule::something(), Some(42));
        });
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use frame_support::construct_runtime;

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        MyPallet: my_pallet,
    }
);
```

## Debugging and Troubleshooting

### Common Issues and Solutions

1. **Weight Calculation Errors**
   ```rust
   // Always benchmark your extrinsics
   #[pallet::weight(T::WeightInfo::do_something())]
   pub fn do_something(origin: OriginFor<T>) -> DispatchResult { ... }
   ```

2. **Storage Migration**
   ```rust
   #[pallet::storage_version(STORAGE_VERSION)]
   pub struct Pallet<T>(_);
   
   pub mod migrations {
       pub fn migrate_to_v2<T: Config>() -> Weight { ... }
   }
   ```

3. **Debugging Macros**
   ```bash
   # Expand macros to see generated code
   cargo expand --package my-pallet
   ```

## Best Practices

1. **Always validate inputs** - Never trust external data
2. **Use proper error handling** - Return descriptive errors
3. **Emit events** - For off-chain monitoring and indexing
4. **Benchmark everything** - Accurate weights prevent DoS
5. **Write comprehensive tests** - Unit, integration, and fuzzing
6. **Document your code** - Especially public APIs

## Advanced Topics

### Cross-Chain Messaging (XCM)
For cross-chain communication patterns, see the dedicated XCM resource.

### Offchain Workers
```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn offchain_worker(block_number: T::BlockNumber) {
        // Perform off-chain computations
        let result = Self::fetch_external_data();
        
        // Submit transaction back on-chain
        let call = Call::submit_data { data: result };
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into());
    }
}
```

## External Resources

### Official Documentation
- **Substrate Documentation**: https://docs.substrate.io/
- **Substrate API Reference**: https://paritytech.github.io/polkadot-sdk/master/
- **FRAME Documentation**: https://docs.substrate.io/reference/frame-pallets/

### Learning Resources
- **Substrate Tutorials**: https://docs.substrate.io/tutorials/
- **Substrate How-to Guides**: https://docs.substrate.io/how-to-guides/
- **Polkadot Wiki**: https://wiki.polkadot.network/
- **Web3 Foundation Research**: https://research.web3.foundation/

### Community and Support
- **Substrate StackExchange**: https://substrate.stackexchange.com/
- **Substrate GitHub**: https://github.com/paritytech/polkadot-sdk
- **Element Chat**: https://matrix.to/#/#substrate-technical:matrix.org
- **Polkadot Forum**: https://forum.polkadot.network/

### Development Tools
- **Substrate Playground**: https://playground.substrate.dev/
- **Polkadot JS Apps**: https://polkadot.js.org/apps/
- **Substrate Sidecar**: https://github.com/paritytech/substrate-api-sidecar
- **Substrate Archive**: https://github.com/paritytech/substrate-archive

## Important Notes for AI Agents

When working with Substrate code:
1. **Always check the Substrate version** - APIs change between versions
2. **Use official polkadot-sdk crates** - Avoid deprecated substrate-* crates
3. **Search for examples in the polkadot-sdk repo** - Real implementations are the best reference
4. **Check Substrate StackExchange** - Common issues are often already answered
5. **Reference the upgrade guides** - When dealing with version migrations

If you encounter unfamiliar patterns or need more specific information:
- Search the official documentation at https://docs.substrate.io/
- Look for examples in https://github.com/paritytech/polkadot-sdk
- Check the Substrate StackExchange for similar questions
- Refer to the API documentation at https://paritytech.github.io/polkadot-sdk/master/