# Polkadot SDK

The Polkadot SDK (formerly Substrate) is a comprehensive blockchain development framework that powers the Polkadot ecosystem. This resource provides extensive documentation about the SDK's architecture, components, and usage patterns to help AI agents work effectively with Polkadot-based projects.

## Overview

The Polkadot SDK is a modular and extensible framework for building application-specific blockchains. It provides:
- **FRAME**: Framework for Runtime Aggregation of Modularized Entities
- **Cumulus**: Tools for building Polkadot parachains
- **XCM**: Cross-consensus messaging format
- **Client libraries**: For interacting with chains
- **Development tools**: Testing, benchmarking, and deployment utilities

## Repository Structure

The Polkadot SDK is organized as a monorepo with the following key components:

```
polkadot-sdk/
├── substrate/           # Core blockchain framework
│   ├── client/         # Client-side components
│   ├── frame/          # Runtime framework and pallets
│   ├── primitives/     # Core types and traits
│   └── utils/          # Utility libraries
├── polkadot/           # Relay chain implementation
│   ├── runtime/        # Relay chain runtimes
│   ├── node/           # Node implementation
│   └── xcm/            # Cross-chain messaging
├── cumulus/            # Parachain development framework
│   ├── client/         # Parachain client components
│   ├── pallets/        # Parachain-specific pallets
│   └── parachains/     # Example parachains
└── bridges/            # Bridge implementations
```

## Core Components

### FRAME System

FRAME is the runtime development framework consisting of:

#### System Pallets
```rust
// Essential pallets that most runtimes need
use frame_system;           // Core system functionality
use pallet_timestamp;       // Block timestamps
use pallet_balances;        // Native currency handling
use pallet_transaction_payment; // Transaction fees
use pallet_sudo;           // Superuser capabilities (dev only)
```

#### Common Pallets
```rust
// Governance
use pallet_democracy;       // Democratic proposals and voting
use pallet_collective;      // Council/committee management
use pallet_treasury;        // Treasury management

// Staking and Consensus
use pallet_staking;         // Nominated Proof-of-Stake
use pallet_session;         // Session key management
use pallet_babe;           // Block production (BABE)
use pallet_grandpa;        // Block finalization (GRANDPA)

// Assets and NFTs
use pallet_assets;         // Fungible assets
use pallet_uniques;        // Non-fungible tokens
use pallet_nfts;          // Advanced NFT features

// Smart Contracts
use pallet_contracts;      // Wasm smart contracts
use pallet_evm;           // Ethereum compatibility
```

### Runtime Construction

```rust
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64, Everything},
    weights::Weight,
};
use sp_runtime::{
    create_runtime_str, generic,
    traits::{BlakeTwo256, IdentityLookup},
};

// Define runtime parameters
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const Version: RuntimeVersion = VERSION;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(Weight::from_ref_time(2_000_000_000_000));
}

// Configure system pallet
impl frame_system::Config for Runtime {
    type BaseCallFilter = Everything;
    type BlockWeights = BlockWeights;
    type BlockLength = ();
    type DbWeight = RocksDbWeight;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

// Construct the runtime
construct_runtime!(
    pub struct Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        // Add more pallets as needed
    }
);
```

## Client Architecture

### Node Components

```rust
// Service builder pattern
use sc_service::{PartialComponents, TaskManager};
use sc_client_api::{ExecutorProvider, RemoteBackend};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

pub fn new_partial(
    config: &Configuration,
) -> Result<PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    sc_consensus::DefaultImportQueue<Block, FullClient>,
    sc_transaction_pool::FullPool<Block, FullClient>,
    (
        impl Fn(
            sc_rpc::DenyUnsafe,
            sc_rpc_server::RpcModule<()>,
        ) -> Result<sc_rpc_server::RpcModule<()>, sc_service::Error>,
        (
            sc_consensus_babe::BabeBlockImport<Block, FullClient, FullGrandpaBlockImport>,
            sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            sc_consensus_babe::BabeLink<Block>,
        ),
        sc_consensus_grandpa::SharedVoterState,
    ),
>, ServiceError> {
    // Implementation details...
}
```

### RPC Interface

```rust
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;

#[rpc(client, server)]
pub trait CustomApi<BlockHash> {
    #[method(name = "custom_getInfo")]
    async fn get_info(&self, at: Option<BlockHash>) -> RpcResult<String>;
}

pub struct CustomRpc<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, Block> CustomApiServer<<Block as BlockT>::Hash> for CustomRpc<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: CustomRuntimeApi<Block>,
{
    async fn get_info(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<String> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(||
            self.client.info().best_hash
        );
        
        api.get_custom_info(at)
            .map_err(|e| jsonrpsee::core::Error::Call(
                jsonrpsee::types::error::CallError::Custom(
                    jsonrpsee::types::error::ErrorObject::owned(
                        1,
                        "Unable to get info",
                        Some(e.to_string()),
                    )
                )
            ))
    }
}
```

## Storage Patterns

### Storage Types and Usage

```rust
use frame_support::pallet_prelude::*;

#[pallet::storage]
#[pallet::getter(fn simple_value)]
pub type SimpleValue<T> = StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
#[pallet::getter(fn user_balance)]
pub type UserBalance<T: Config> = StorageMap<
    _,
    Blake2_128Concat,  // Hasher
    T::AccountId,      // Key
    Balance,           // Value
    ValueQuery,        // Query type
>;

#[pallet::storage]
pub type DoubleMapExample<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,      // First key
    Twox64Concat,      // Second hasher
    ItemId,            // Second key
    ItemDetails,       // Value
    OptionQuery,       // Returns Option<ItemDetails>
>;

#[pallet::storage]
pub type ComplexStorage<T: Config> = StorageNMap<
    _,
    (
        NMapKey<Blake2_128Concat, T::AccountId>,
        NMapKey<Twox64Concat, CategoryId>,
        NMapKey<Identity, ItemId>,
    ),
    ItemMetadata,
    ValueQuery,
>;
```

### Storage Migrations

```rust
use frame_support::traits::OnRuntimeUpgrade;
use frame_support::weights::Weight;

pub mod v2 {
    use super::*;
    
    pub struct MigrateToV2<T>(PhantomData<T>);
    
    impl<T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
        fn on_runtime_upgrade() -> Weight {
            let current_version = StorageVersion::get::<Pallet<T>>();
            
            if current_version < 2 {
                // Perform migration
                let mut translated = 0u64;
                
                OldStorage::<T>::translate::<OldType, _>(|_key, old_value| {
                    translated += 1;
                    Some(migrate_to_new_type(old_value))
                });
                
                StorageVersion::new(2).put::<Pallet<T>>();
                
                T::DbWeight::get().reads_writes(translated + 1, translated + 1)
            } else {
                Weight::zero()
            }
        }
        
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            let count = OldStorage::<T>::iter().count();
            Ok(count.encode())
        }
        
        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
            let old_count: usize = Decode::decode(&mut &state[..])
                .map_err(|_| "Failed to decode state")?;
            let new_count = NewStorage::<T>::iter().count();
            
            ensure!(old_count == new_count, "Migration failed");
            Ok(())
        }
    }
}
```

## Parachain Development with Cumulus

### Parachain Runtime

```rust
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use cumulus_primitives_core::ParaId;

parameter_types! {
    pub const ParachainId: ParaId = ParaId::new(2000);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnSystemEvent = ();
    type SelfParaId = ParachainId;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

// XCM Configuration
impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = ();
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type WeightInfo = ();
    type PriceForSiblingDelivery = ();
}
```

### Collator Node

```rust
use cumulus_client_service::{
    prepare_node_config, start_collator, start_full_node,
    StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;

pub async fn start_parachain_node(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    para_id: ParaId,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(
    TaskManager,
    Arc<TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>>,
)> {
    let params = new_partial(&parachain_config)?;
    
    let (relay_chain_interface, collator_key) = build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
        hwbench.clone(),
    ).await?;
    
    let params = StartCollatorParams {
        para_id,
        block_status: client.clone(),
        announce_block,
        client: client.clone(),
        task_manager: &mut task_manager,
        relay_chain_interface,
        spawner,
        parachain_consensus,
        import_queue,
        collator_key,
        relay_chain_slot_duration,
    };
    
    start_collator(params).await?;
    
    Ok((task_manager, client))
}
```

## Benchmarking and Weights

### Writing Benchmarks

```rust
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    do_something {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let something = s.into();
    }: _(RawOrigin::Signed(caller), something)
    verify {
        assert_eq!(Something::<T>::get(), Some(something));
    }
    
    cause_error {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let bad_origin = RawOrigin::Root;
    }: _(bad_origin, s.into())
    verify {
        assert!(Something::<T>::get().is_none());
    }
    
    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
```

### Weight Calculation

```rust
use frame_support::weights::{Weight, constants::RocksDbWeight};

pub trait WeightInfo {
    fn do_something(s: u32) -> Weight;
    fn cause_error(s: u32) -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn do_something(s: u32) -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(Weight::from_parts(100_000, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn cause_error(_s: u32) -> Weight {
        Weight::from_parts(5_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
    }
}
```

## Testing Patterns

### Mock Runtime

```rust
use frame_support::{
    traits::{ConstU16, ConstU64},
    weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        TemplateModule: pallet_template,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
```

### Integration Tests

```rust
#[test]
fn test_complete_workflow() {
    new_test_ext().execute_with(|| {
        // Setup
        System::set_block_number(1);
        
        // Test initial state
        assert_eq!(Something::<Test>::get(), None);
        
        // Perform action
        assert_ok!(TemplateModule::do_something(
            RuntimeOrigin::signed(1),
            42
        ));
        
        // Verify state change
        assert_eq!(Something::<Test>::get(), Some(42));
        
        // Verify event
        System::assert_last_event(
            Event::SomethingStored { 
                something: 42, 
                who: 1 
            }.into()
        );
    });
}
```

## Common Development Tasks

### Adding a New Pallet

1. **Create pallet structure**:
```bash
cd pallets
cargo new pallet-example --lib
```

2. **Update Cargo.toml**:
```toml
[dependencies]
codec = { package = "parity-scale-codec", version = "3.0.0", default-features = false }
scale-info = { version = "2.0.0", default-features = false }
frame-support = { version = "4.0.0-dev", default-features = false }
frame-system = { version = "4.0.0-dev", default-features = false }

[features]
default = ["std"]
std = [
    "codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
]
```

3. **Implement pallet logic**
4. **Add to runtime**
5. **Run benchmarks**
6. **Write tests**

### Upgrading Runtime

```rust
use sp_runtime::{create_runtime_str, impl_opaque_keys};

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("node-template"),
    impl_name: create_runtime_str!("node-template"),
    authoring_version: 1,
    spec_version: 102,  // Increment for logic changes
    impl_version: 1,    // Increment for implementation changes
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

// Migration module
pub mod migrations {
    use super::*;
    
    pub type Upgrade = (
        pallet_balances::migration::MigrateToTrackInactive<Runtime, CheckAccount>,
        my_pallet::migration::v2::MigrateToV2<Runtime>,
    );
}

// In runtime construction
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    migrations::Upgrade,  // Apply migrations
>;
```

## Best Practices

### Error Handling

```rust
#[pallet::error]
pub enum Error<T> {
    /// The value was too low
    ValueTooLow,
    /// The account has insufficient balance
    InsufficientBalance,
    /// The operation would cause an overflow
    Overflow,
    /// The item was not found
    ItemNotFound,
    /// The operation is not allowed for this account
    NotAllowed,
}

// Usage
ensure!(value >= T::MinimumValue::get(), Error::<T>::ValueTooLow);
```

### Event Design

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    /// A new item was created. [owner, item_id]
    ItemCreated(T::AccountId, ItemId),
    
    /// An item was transferred. [from, to, item_id]
    ItemTransferred(T::AccountId, T::AccountId, ItemId),
    
    /// An item was burned. [owner, item_id]
    ItemBurned(T::AccountId, ItemId),
}
```

### Security Considerations

1. **Always validate inputs**
2. **Use safe math operations**
3. **Implement proper access control**
4. **Avoid unbounded operations**
5. **Benchmark all extrinsics**

```rust
// Safe math
let new_balance = old_balance
    .checked_add(amount)
    .ok_or(Error::<T>::Overflow)?;

// Access control
ensure!(
    Self::is_owner(&who, &item_id)?,
    Error::<T>::NotOwner
);

// Bounded iterations
let items: BoundedVec<_, T::MaxItems> = 
    items.try_into()
    .map_err(|_| Error::<T>::TooManyItems)?;
```

## Debugging and Development Tools

### Logging

```rust
use log::{debug, info, warn, error};

#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn do_something(origin: OriginFor<T>, value: u32) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        debug!(target: "my-pallet", "do_something called by {:?} with value {}", who, value);
        
        // Logic here
        
        info!(target: "my-pallet", "Successfully processed value {} for {:?}", value, who);
        
        Ok(())
    }
}
```

### Try-Runtime

```rust
#[cfg(feature = "try-runtime")]
impl<T: Config> Pallet<T> {
    fn try_state(_n: BlockNumber) -> Result<(), &'static str> {
        // Verify storage invariants
        ensure!(
            Something::<T>::get().unwrap_or(0) < 1000,
            "Something value too high"
        );
        
        Ok(())
    }
}
```

## External Resources

### Official Documentation
- **Polkadot SDK Docs**: https://paritytech.github.io/polkadot-sdk/master/
- **Substrate Docs**: https://docs.substrate.io/
- **Polkadot Wiki**: https://wiki.polkadot.network/

### Source Code
- **GitHub Repository**: https://github.com/paritytech/polkadot-sdk
- **Releases**: https://github.com/paritytech/polkadot-sdk/releases

### Learning Resources
- **Substrate Tutorials**: https://docs.substrate.io/tutorials/
- **Polkadot Blockchain Academy**: https://polkadot.network/development/blockchain-academy/
- **Web3 Foundation Grants**: https://web3.foundation/grants/

### Community
- **Stack Exchange**: https://substrate.stackexchange.com/
- **Discord**: https://discord.gg/polkadot
- **Element/Matrix**: https://matrix.to/#/#substrate-technical:matrix.org
- **Forum**: https://forum.polkadot.network/

### Development Tools
- **Substrate Playground**: https://playground.substrate.dev/
- **Contracts UI**: https://contracts-ui.substrate.io/
- **Polkadot JS Apps**: https://polkadot.js.org/apps/
- **Subscan Explorer**: https://subscan.io/

## Important Notes for AI Agents

When working with Polkadot SDK:
1. **Version awareness**: Always check the SDK version as APIs evolve
2. **Crate naming**: Use `polkadot-sdk` crates, not old `substrate-*` names
3. **Example code**: Best examples are in the SDK repository itself
4. **Documentation**: Check both rustdocs and the docs.substrate.io site
5. **Testing**: Always write comprehensive tests for runtime logic

For specific questions:
- Search the polkadot-sdk GitHub repository for real examples
- Check Substrate StackExchange for common issues
- Look at parachain implementations for production patterns
- Consult the Polkadot Wiki for conceptual understanding

Remember: The ecosystem is actively developed. Always verify information against the latest documentation and consider checking the source code when in doubt.