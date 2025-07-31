# Scaffold Pallet

## Description

Generate pallet structure and implementation templates

## Arguments

- pallet_description: description for the pallet

## Prompt

```
Create a complete Substrate pallet scaffold based on the following description:
<pallet_description>

## Project Structure
Create the following file structure:
```
pallets/<pallet_name>/
├── Cargo.toml
├── src/
│   ├── lib.rs         # Main pallet logic
│   ├── mock.rs        # Test runtime setup
│   ├── tests.rs       # Unit tests
│   ├── benchmarking.rs # Benchmarks
│   └── weights.rs     # Auto-generated weights
└── README.md          # Pallet documentation
```

## Implementation Requirements

### 1. Core Pallet Structure (`src/lib.rs`)
```rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
        
        // Add other configuration parameters based on <pallet_description>
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Storage items based on pallet requirements
    #[pallet::storage]
    #[pallet::getter(fn example_storage)]
    pub type ExampleStorage<T> = StorageValue<_, u32, ValueQuery>;

    /// Events emitted by the pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation
        SomethingHappened { who: T::AccountId, value: u32 },
    }

    /// Errors that can be returned by this pallet
    #[pallet::error]
    pub enum Error<T> {
        /// Error documentation
        InvalidInput,
        InsufficientPermission,
        // Add errors based on <pallet_description>
    }

    /// Dispatchable functions (extrinsics)
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Documentation for the extrinsic
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::example_extrinsic())]
        pub fn example_extrinsic(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Validation
            ensure!(value > 0, Error::<T>::InvalidInput);
            
            // State changes
            <ExampleStorage<T>>::put(value);
            
            // Emit event
            Self::deposit_event(Event::SomethingHappened { who, value });
            
            Ok(())
        }
    }

    /// Helper functions (private)
    impl<T: Config> Pallet<T> {
        fn helper_function() -> Result<(), Error<T>> {
            // Implementation
            Ok(())
        }
    }
}

/// Weight information trait
pub trait WeightInfo {
    fn example_extrinsic() -> Weight;
}

/// Default weight implementation
impl WeightInfo for () {
    fn example_extrinsic() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
```

### 2. Mock Runtime (`src/mock.rs`)
```rust
use crate as pallet_template;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        TemplateModule: pallet_template,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    // System config implementation
}

impl pallet_template::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
```

### 3. Unit Tests (`src/tests.rs`)
Include tests for:
- ✅ Happy path scenarios
- ❌ Error conditions
- 🔒 Permission checks
- 📊 State changes
- 📢 Event emissions

```rust
use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn example_extrinsic_works() {
    new_test_ext().execute_with(|| {
        // Arrange
        let caller = 1;
        let value = 42;
        
        // Act
        assert_ok!(TemplateModule::example_extrinsic(
            RuntimeOrigin::signed(caller),
            value
        ));
        
        // Assert
        assert_eq!(TemplateModule::example_storage(), value);
        System::assert_last_event(
            Event::SomethingHappened { who: caller, value }.into()
        );
    });
}

#[test]
fn example_extrinsic_fails_with_invalid_input() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TemplateModule::example_extrinsic(RuntimeOrigin::signed(1), 0),
            Error::<Test>::InvalidInput
        );
    });
}
```

### 4. Benchmarks (`src/benchmarking.rs`)
```rust
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    example_extrinsic {
        let caller: T::AccountId = whitelisted_caller();
        let value = 100u32;
    }: _(RawOrigin::Signed(caller.clone()), value)
    verify {
        assert_eq!(ExampleStorage::<T>::get(), value);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
```

### 5. Cargo.toml
```toml
[package]
name = "pallet-<pallet_name>"
version = "0.1.0"
authors = ["Your Name"]
edition = "2021"

[dependencies]
codec = { package = "parity-scale-codec", version = "3.6.1", default-features = false }
scale-info = { version = "2.10.0", default-features = false, features = ["derive"] }
frame-support = { default-features = false, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
frame-system = { default-features = false, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
frame-benchmarking = { default-features = false, optional = true, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }

[dev-dependencies]
sp-core = { git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
sp-io = { git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
sp-runtime = { git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }

[features]
default = ["std"]
std = [
    "codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
    "frame-benchmarking?/std",
]
runtime-benchmarks = [
    "frame-benchmarking/runtime-benchmarks",
    "frame-support/runtime-benchmarks",
    "frame-system/runtime-benchmarks",
]
```

## Implementation Guidelines

1. **Storage Design**
   - Use appropriate storage types (Value, Map, DoubleMap)
   - Consider storage costs and access patterns
   - Add proper getters with documentation

2. **Error Handling**
   - Define specific, descriptive errors
   - Use `ensure!` for validation
   - Return early on errors

3. **Events**
   - Emit events for all state changes
   - Include relevant data for indexing
   - Document event meanings

4. **Weights**
   - Benchmark all extrinsics
   - Use realistic worst-case scenarios
   - Update weights after changes

5. **Testing**
   - Test all success paths
   - Test all error conditions
   - Test edge cases and boundaries
   - Test event emissions

## References
- Basic pallet structure: https://docs.polkadot.com/develop/parachains/customize-parachain/make-custom-pallet/
- Testing guide: https://docs.polkadot.com/develop/parachains/testing/pallet-testing/
- Benchmarking: https://docs.polkadot.com/develop/parachains/testing/benchmarking/

```