#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame::deps::{
	frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}},
	frame_system,
};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn create_escrow() -> Weight;
	fn release_funds() -> Weight;
	fn refund() -> Weight;
	fn claim_expired() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn create_escrow() -> Weight {
		Weight::from_parts(50_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(5))
	}

	fn release_funds() -> Weight {
		Weight::from_parts(45_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}

	fn refund() -> Weight {
		Weight::from_parts(40_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}

	fn claim_expired() -> Weight {
		Weight::from_parts(42_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

impl WeightInfo for () {
	fn create_escrow() -> Weight {
		Weight::from_parts(50_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(5))
	}

	fn release_funds() -> Weight {
		Weight::from_parts(45_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(3))
	}

	fn refund() -> Weight {
		Weight::from_parts(40_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(3))
	}

	fn claim_expired() -> Weight {
		Weight::from_parts(42_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3523))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
}