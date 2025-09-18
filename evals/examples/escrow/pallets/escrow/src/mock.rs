use crate as pallet_escrow;
use frame::{
	deps::{
		frame_support::{
			parameter_types, 
			weights::constants::RocksDbWeight,
			PalletId,
		},
		frame_system::GenesisConfig,
	},
	prelude::*,
	runtime::prelude::*,
	testing_prelude::*,
};
use polkadot_sdk::{pallet_balances, pallet_timestamp};

// Configure a mock runtime to test the pallet.
#[frame_construct_runtime]
mod test_runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	
	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances;
	
	#[runtime::pallet_index(2)]
	pub type Timestamp = pallet_timestamp;
	
	#[runtime::pallet_index(3)]
	pub type Escrow = pallet_escrow;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Nonce = u64;
	type Block = MockBlock<Test>;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
	type AccountData = pallet_balances::AccountData<u128>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = u128;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1>;
	type WeightInfo = ();
}

parameter_types! {
	pub const EscrowPalletId: PalletId = PalletId(*b"escrow  ");
	pub const MinimumEscrowAmount: u128 = 10;
	pub const MaximumEscrowAmount: u128 = 1_000_000;
	pub const MinimumDeadline: u64 = 10;
	pub const MaximumDeadline: u64 = 100_000;
	pub const MaxDescriptionLength: u32 = 500;
	pub const MaxActiveEscrowsPerUser: u32 = 100;
	pub const PlatformFeePercent: u32 = 100;
}

impl pallet_escrow::Config for Test {
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
	type WeightInfo = ();
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

pub fn new_test_ext() -> TestState {
	let mut t = GenesisConfig::<Test>::default().build_storage().unwrap();
	
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 1_000_000),
			(BOB, 1_000_000),
			(CHARLIE, 1_000_000),
		],
		dev_accounts: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();
	
	let mut ext = TestState::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
	}
}