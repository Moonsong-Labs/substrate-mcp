# XCM Caching

XCM (Cross-Consensus Messaging) is Polkadot's native interoperability protocol that enables communication between different chains in the Polkadot ecosystem. This resource provides comprehensive information about XCM caching strategies, patterns, and implementation details.

## Overview

XCM caching is essential for optimizing cross-chain message processing and reducing redundant computations in parachain runtime. It involves storing frequently accessed XCM-related data to improve performance and reduce on-chain storage reads.

## Core Concepts

### XCM Message Structure
XCM messages consist of instructions that are executed sequentially:
- **Instructions**: Basic operations like `TransferAsset`, `BuyExecution`, `Transact`
- **Assets**: Multi-asset representation for fungible and non-fungible tokens
- **Locations**: Universal way to identify accounts, chains, and assets
- **Junctions**: Building blocks of locations (e.g., `Parachain(id)`, `AccountId32`)

### Caching Strategies

#### 1. Asset Registry Caching
Cache frequently accessed asset information to avoid repeated storage reads:

```rust
use frame_support::storage::StorageMap;
use xcm::latest::{AssetId, MultiLocation};

#[pallet::storage]
pub type CachedAssetLocations<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    AssetId,
    (MultiLocation, BlockNumberFor<T>), // Location and last update block
    OptionQuery,
>;

impl<T: Config> Pallet<T> {
    pub fn get_asset_location(asset_id: AssetId) -> Option<MultiLocation> {
        // Check cache first
        if let Some((location, last_update)) = CachedAssetLocations::<T>::get(&asset_id) {
            let current_block = frame_system::Pallet::<T>::block_number();
            // Cache valid for 100 blocks
            if current_block.saturating_sub(last_update) < 100u32.into() {
                return Some(location);
            }
        }
        
        // Fetch from source and update cache
        let location = Self::fetch_asset_location_from_source(asset_id)?;
        let current_block = frame_system::Pallet::<T>::block_number();
        CachedAssetLocations::<T>::insert(&asset_id, (location, current_block));
        
        Some(location)
    }
}
```

#### 2. XCM Execution Cost Caching
Cache execution costs for common XCM programs:

```rust
use sp_std::collections::btree_map::BTreeMap;
use xcm::latest::{Xcm, Weight};

#[pallet::storage]
pub type XcmExecutionCache<T: Config> = StorageValue<
    _,
    BTreeMap<Blake2_256Hash, (Weight, BalanceOf<T>)>,
    ValueQuery,
>;

pub fn cache_xcm_execution_cost<T: Config>(
    xcm: &Xcm<()>,
    weight: Weight,
    fee: BalanceOf<T>,
) {
    let hash = xcm.using_encoded(blake2_256);
    XcmExecutionCache::<T>::mutate(|cache| {
        // Keep cache size limited
        if cache.len() >= 1000 {
            // Remove oldest entry (BTreeMap maintains order)
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }
        cache.insert(hash, (weight, fee));
    });
}
```

#### 3. Junction Resolution Caching
Cache junction-to-account conversions:

```rust
#[pallet::storage]
pub type JunctionAccountCache<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    MultiLocation, // Origin
    Blake2_128Concat,
    Junction,      // Target junction
    T::AccountId,  // Resolved account
    OptionQuery,
>;

impl<T: Config> Pallet<T> {
    pub fn resolve_junction_to_account(
        origin: &MultiLocation,
        junction: &Junction,
    ) -> Result<T::AccountId, Error<T>> {
        // Check cache
        if let Some(account) = JunctionAccountCache::<T>::get(origin, junction) {
            return Ok(account);
        }
        
        // Expensive resolution
        let account = T::LocationToAccountId::convert_location(junction)
            .ok_or(Error::<T>::AccountResolutionFailed)?;
        
        // Cache result
        JunctionAccountCache::<T>::insert(origin, junction, &account);
        
        Ok(account)
    }
}
```

## Implementation Patterns

### Efficient XCM Handler with Caching

```rust
use xcm_executor::traits::TransactAsset;

pub struct CachedXcmHandler<T: Config> {
    _phantom: PhantomData<T>,
}

impl<T: Config> TransactAsset for CachedXcmHandler<T> {
    fn deposit_asset(
        what: &MultiAsset,
        who: &MultiLocation,
        context: &XcmContext,
    ) -> XcmResult {
        // Use cached conversions
        let beneficiary = T::CachedLocationConverter::convert_location(who)
            .ok_or(XcmError::InvalidLocation)?;
        
        let asset_id = T::CachedAssetConverter::convert_asset(what)
            .ok_or(XcmError::AssetNotFound)?;
        
        // Perform deposit with cached data
        T::Assets::mint_into(asset_id, &beneficiary, what.amount())
            .map_err(|_| XcmError::FailedToTransactAsset)?;
        
        Ok(())
    }
}
```

### Cache Invalidation Strategies

```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight {
        // Periodic cache cleanup every 1000 blocks
        if (n % 1000u32.into()).is_zero() {
            // Clear old cache entries
            CachedAssetLocations::<T>::translate(
                |_key, (location, last_update): (MultiLocation, BlockNumberFor<T>)| {
                    if n.saturating_sub(last_update) > 10000u32.into() {
                        None // Remove stale entries
                    } else {
                        Some((location, last_update))
                    }
                },
            );
            
            T::DbWeight::get().reads_writes(100, 50) // Estimate
        } else {
            Weight::zero()
        }
    }
}
```

### Memory-Efficient LRU Cache

```rust
use sp_std::collections::vec_deque::VecDeque;

pub struct LruXcmCache<T: Config> {
    items: VecDeque<(XcmHash, CachedXcmData)>,
    max_size: usize,
}

impl<T: Config> LruXcmCache<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn get(&mut self, hash: &XcmHash) -> Option<&CachedXcmData> {
        let position = self.items.iter().position(|(h, _)| h == hash)?;
        
        // Move to front (most recently used)
        let (hash, data) = self.items.remove(position)?;
        self.items.push_front((hash, data));
        
        self.items.front().map(|(_, data)| data)
    }
    
    pub fn insert(&mut self, hash: XcmHash, data: CachedXcmData) {
        // Remove if exists
        if let Some(pos) = self.items.iter().position(|(h, _)| h == &hash) {
            self.items.remove(pos);
        }
        
        // Add to front
        self.items.push_front((hash, data));
        
        // Maintain size limit
        while self.items.len() > self.max_size {
            self.items.pop_back();
        }
    }
}
```

## Common XCM Patterns with Caching

### Reserve Asset Transfer with Cached Routes

```rust
pub fn transfer_reserve_asset_cached<T: Config>(
    dest: MultiLocation,
    beneficiary: MultiLocation,
    assets: MultiAssets,
) -> DispatchResult {
    // Check cached route
    let route = T::RouteCache::get_route(&dest)
        .unwrap_or_else(|| {
            let new_route = compute_optimal_route(&dest);
            T::RouteCache::insert_route(&dest, &new_route);
            new_route
        });
    
    let message = Xcm(vec![
        WithdrawAsset(assets.clone()),
        InitiateReserveWithdraw {
            assets: All.into(),
            reserve: route.reserve_location,
            xcm: Xcm(vec![
                BuyExecution { 
                    fees: assets.get(0).cloned().unwrap_or_default(),
                    weight_limit: Unlimited,
                },
                DepositAsset {
                    assets: All.into(),
                    beneficiary,
                },
            ]),
        },
    ]);
    
    T::XcmRouter::send_xcm(dest, message).map_err(|_| Error::<T>::SendFailed)?;
    
    Ok(())
}
```

### Cached Fee Calculation

```rust
pub struct CachedFeeCalculator<T: Config> {
    cache: RefCell<BTreeMap<(MultiLocation, MultiAsset), Balance>>,
}

impl<T: Config> FeeCalculator for CachedFeeCalculator<T> {
    fn calculate_fee(
        dest: &MultiLocation,
        asset: &MultiAsset,
        message_weight: Weight,
    ) -> Option<Balance> {
        let cache_key = (dest.clone(), asset.clone());
        
        // Check cache
        if let Some(fee) = self.cache.borrow().get(&cache_key) {
            return Some(*fee);
        }
        
        // Calculate fee
        let base_fee = T::BaseXcmFee::get();
        let weight_fee = T::WeightToFee::weight_to_fee(&message_weight);
        let destination_multiplier = T::DestinationMultiplier::get_multiplier(dest);
        
        let total_fee = base_fee
            .saturating_add(weight_fee)
            .saturating_mul(destination_multiplier);
        
        // Update cache
        self.cache.borrow_mut().insert(cache_key, total_fee);
        
        Some(total_fee)
    }
}
```

## Performance Optimization Tips

### 1. Batch Cache Updates
```rust
pub fn batch_update_asset_cache<T: Config>(
    updates: Vec<(AssetId, MultiLocation)>,
) -> Weight {
    let current_block = frame_system::Pallet::<T>::block_number();
    
    // Use batch storage operations
    CachedAssetLocations::<T>::insert_many(
        updates.into_iter().map(|(id, location)| {
            (id, (location, current_block))
        })
    );
    
    T::DbWeight::get().writes(updates.len() as u64)
}
```

### 2. Lazy Loading with Cache
```rust
pub struct LazyXcmCache<T: Config> {
    inner: OnceCell<BTreeMap<XcmHash, XcmMetadata>>,
}

impl<T: Config> LazyXcmCache<T> {
    pub fn get_or_init(&self) -> &BTreeMap<XcmHash, XcmMetadata> {
        self.inner.get_or_init(|| {
            // Load from storage only when first accessed
            XcmMetadataStorage::<T>::get()
        })
    }
}
```

### 3. Probabilistic Cache Eviction
```rust
pub fn maybe_evict_cache_entry<T: Config>(block: BlockNumberFor<T>) {
    // Evict with 1% probability to spread load
    let hash = T::Hashing::hash_of(&block);
    let random = u32::from_le_bytes([hash.as_ref()[0], hash.as_ref()[1], hash.as_ref()[2], hash.as_ref()[3]]);
    
    if random % 100 == 0 {
        // Evict oldest entries
        perform_cache_maintenance::<T>();
    }
}
```

## Testing Cache Behavior

### Unit Tests
```rust
#[test]
fn test_xcm_cache_eviction() {
    new_test_ext().execute_with(|| {
        let mut cache = LruXcmCache::<Test>::new(3);
        
        // Fill cache
        cache.insert(hash1(), data1());
        cache.insert(hash2(), data2());
        cache.insert(hash3(), data3());
        
        // This should evict hash1
        cache.insert(hash4(), data4());
        
        assert!(cache.get(&hash1()).is_none());
        assert!(cache.get(&hash4()).is_some());
    });
}
```

### Benchmarking Cached vs Non-Cached
```rust
benchmarks! {
    resolve_asset_with_cache {
        let asset_id = AssetId::Concrete(MultiLocation::parent());
        // Prime the cache
        Pallet::<T>::get_asset_location(asset_id.clone());
    }: {
        Pallet::<T>::get_asset_location(asset_id)
    }
    
    resolve_asset_without_cache {
        let asset_id = AssetId::Concrete(MultiLocation::new(2, X1(Parachain(2000))));
        // Ensure not in cache
        CachedAssetLocations::<T>::remove(&asset_id);
    }: {
        Pallet::<T>::fetch_asset_location_from_source(asset_id)
    }
}
```

## Common Pitfalls and Solutions

### 1. Cache Invalidation Issues
**Problem**: Stale cache entries leading to incorrect behavior

**Solution**: Implement TTL (Time To Live) and event-based invalidation
```rust
#[pallet::event]
pub enum Event<T: Config> {
    AssetRegistryUpdated { asset_id: AssetId },
}

// Invalidate cache on registry update
fn on_asset_registry_update(asset_id: AssetId) {
    CachedAssetLocations::<T>::remove(&asset_id);
    Self::deposit_event(Event::AssetRegistryUpdated { asset_id });
}
```

### 2. Memory Bloat
**Problem**: Unbounded cache growth

**Solution**: Implement size limits and eviction policies
```rust
#[pallet::constant]
type MaxCacheEntries: Get<u32>;

fn ensure_cache_bounds<T: Config>() {
    let current_size = CachedAssetLocations::<T>::iter().count();
    if current_size > T::MaxCacheEntries::get() as usize {
        // Evict oldest entries
        evict_oldest_entries::<T>(current_size - T::MaxCacheEntries::get() as usize);
    }
}
```

### 3. Cache Warming
**Problem**: Cold cache after runtime upgrade

**Solution**: Implement cache warming in migration
```rust
pub mod migration {
    pub fn warm_cache_v2<T: Config>() -> Weight {
        // Pre-populate critical cache entries
        let critical_assets = vec![
            (AssetId::Native, MultiLocation::here()),
            // Add more critical assets
        ];
        
        for (asset_id, location) in critical_assets {
            CachedAssetLocations::<T>::insert(
                &asset_id,
                (location, frame_system::Pallet::<T>::block_number()),
            );
        }
        
        T::DbWeight::get().writes(critical_assets.len() as u64)
    }
}
```

## External Resources

### Official Documentation
- **XCM Format Specification**: https://github.com/paritytech/xcm-format
- **XCM Documentation**: https://wiki.polkadot.network/docs/learn-xcm
- **Polkadot SDK XCM**: https://paritytech.github.io/polkadot-sdk/master/xcm/index.html

### Implementation References
- **Cumulus XCM**: https://github.com/paritytech/polkadot-sdk/tree/master/cumulus/pallets/xcmp-queue
- **Asset Hub Implementation**: https://github.com/paritytech/polkadot-sdk/tree/master/cumulus/parachains/runtimes/assets
- **XCM Executor**: https://github.com/paritytech/polkadot-sdk/tree/master/polkadot/xcm/xcm-executor

### Learning Resources
- **XCM Workshop**: https://www.youtube.com/watch?v=5cgq5jOZx9g
- **Sub0 XCM Workshop**: https://substrate.io/ecosystem/substrate-events/sub0-2022/
- **XCM by Example**: https://github.com/paritytech/xcm-by-example

### Tools and Testing
- **XCM Simulator**: https://github.com/paritytech/polkadot-sdk/tree/master/polkadot/xcm/xcm-simulator
- **Chopsticks**: https://github.com/AcalaNetwork/chopsticks (XCM testing tool)
- **XCM Tools**: https://github.com/albertov19/xcm-tools

## Important Notes for AI Agents

When implementing XCM caching:
1. **Always consider cache invalidation** - XCM configurations can change
2. **Implement proper bounds** - Prevent unbounded storage growth
3. **Test with XCM Simulator** - Verify cache behavior in multi-chain scenarios
4. **Monitor cache hit rates** - Add metrics for cache effectiveness
5. **Handle runtime upgrades** - Caches may need migration

For specific implementation questions:
- Check existing parachain implementations in polkadot-sdk
- Look for XCM caching patterns in production parachains
- Consult the Polkadot Forum for community solutions
- Test thoroughly with xcm-simulator for edge cases

Remember: XCM is evolving rapidly. Always check for the latest XCM version and migration guides when implementing caching strategies.