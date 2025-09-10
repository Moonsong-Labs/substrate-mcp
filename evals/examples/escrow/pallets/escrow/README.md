# Escrow Pallet

A decentralized escrow pallet for secure transactions between parties on Substrate-based blockchains.

## Overview

The Escrow Pallet facilitates trustless transactions between buyers and sellers by holding funds in escrow until specific conditions are met. It ensures both parties fulfill their obligations without requiring mutual trust.

## Features

- **Secure Fund Locking**: Buyer's funds are locked in the pallet's secure account
- **Automatic Safeguards**: Timeout mechanisms prevent indefinite fund locks
- **Platform Fees**: Configurable fee structure (default 1%)
- **User Limits**: Maximum active escrows per user to prevent spam
- **Flexible Configuration**: Adjustable minimum/maximum amounts and deadlines

## User Operations

### Create Escrow
Buyers can create an escrow transaction specifying:
- Seller address
- Payment amount
- Transaction description (up to 500 characters)
- Deadline (between 1 hour and 90 days)

### Release Funds
Buyers can release funds to the seller when satisfied with goods/services.

### Request Refund
Sellers can initiate refunds if unable to fulfill orders.

### Claim Expired
Buyers can reclaim funds after the deadline passes without resolution.

## Integration

### 1. Add to Workspace

In your workspace `Cargo.toml`:

```toml
[workspace.dependencies]
pallet-escrow = { path = "./pallets/escrow", default-features = false }
```

### 2. Add to Runtime

In `runtime/Cargo.toml`:

```toml
[dependencies]
pallet-escrow = { workspace = true }

[features]
std = [
    # ... other pallets
    "pallet-escrow/std",
]
runtime-benchmarks = [
    # ... other pallets
    "pallet-escrow/runtime-benchmarks",
]
try-runtime = [
    # ... other pallets
    "pallet-escrow/try-runtime",
]
```

### 3. Configure Runtime

In `runtime/src/configs/mod.rs` or equivalent:

```rust
use frame_support::{parameter_types, PalletId};

parameter_types! {
    pub const EscrowPalletId: PalletId = PalletId(*b"escrow  ");
    pub const MinimumEscrowAmount: Balance = 10 * EXISTENTIAL_DEPOSIT;
    pub const MaximumEscrowAmount: Balance = 1_000_000 * UNIT;
    pub const MinimumDeadline: BlockNumber = 600; // ~1 hour at 6s blocks
    pub const MaximumDeadline: BlockNumber = 1_296_000; // ~90 days at 6s blocks
    pub const MaxDescriptionLength: u32 = 500;
    pub const MaxActiveEscrowsPerUser: u32 = 100;
    pub const PlatformFeePercent: u32 = 100; // 1% = 100 basis points
}

impl pallet_escrow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type PalletId = EscrowPalletId;
    type MinimumEscrowAmount = MinimumEscrowAmount;
    type MaximumEscrowAmount = MaximumEscrowAmount;
    type MinimumDeadline = MinimumDeadline;
    type MaximumDeadline = MaximumDeadline;
    type MaxDescriptionLength = MaxDescriptionLength;
    type MaxActiveEscrowsPerUser = MaxActiveEscrowsPerUser;
    type PlatformFeePercent = PlatformFeePercent;
    type WeightInfo = pallet_escrow::weights::SubstrateWeight<Runtime>;
}
```

### 4. Add to Construct Runtime

In `runtime/src/lib.rs`:

```rust
#[frame_support::runtime]
mod runtime {
    // ... other pallets

    #[runtime::pallet_index(51)]
    pub type Escrow = pallet_escrow;
}
```

## Testing

Run the pallet tests:

```bash
cargo test -p pallet-escrow
```

## Benchmarking

Generate weights for your runtime:

```bash
cargo build --release --features runtime-benchmarks
./target/release/parachain-template-node benchmark pallet \
    --chain dev \
    --pallet pallet_escrow \
    --extrinsic "*" \
    --steps 50 \
    --repeat 20 \
    --output pallets/escrow/src/weights.rs \
    --template .maintain/frame-weight-template.hbs
```

## Configuration Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `MinimumEscrowAmount` | Minimum amount for an escrow | 10 units |
| `MaximumEscrowAmount` | Maximum amount for an escrow | 1,000,000 units |
| `MinimumDeadline` | Minimum deadline (in blocks) | 600 blocks (~1 hour) |
| `MaximumDeadline` | Maximum deadline (in blocks) | 1,296,000 blocks (~90 days) |
| `MaxDescriptionLength` | Maximum description length | 500 characters |
| `MaxActiveEscrowsPerUser` | Maximum active escrows per user | 100 |
| `PlatformFeePercent` | Platform fee in basis points | 100 (1%) |

## Security Considerations

1. **Fund Safety**: Funds are held in reserve, ensuring they cannot be spent elsewhere
2. **Authorization**: Only authorized parties can perform actions on escrows
3. **Rate Limiting**: Maximum active escrows per user prevents spam
4. **Minimum Amount**: Prevents dust attacks and ensures meaningful transactions

## License

MIT-0