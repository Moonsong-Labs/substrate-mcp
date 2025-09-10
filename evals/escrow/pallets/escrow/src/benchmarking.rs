#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame::{
	deps::frame_support::{assert_ok, traits::Currency},
	deps::frame_system::RawOrigin,
	benchmarking::v2::*,
	traits::fungible::Inspect,
};
use scale_info::prelude::{vec, vec::Vec};

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_escrow() {
		let buyer: T::AccountId = account("buyer", 0, 0);
		let seller: T::AccountId = account("seller", 0, 0);
		let amount = T::MinimumEscrowAmount::get();
		let description = vec![0u8; T::MaxDescriptionLength::get() as usize];
		let deadline = frame_system::Pallet::<T>::block_number() + T::MinimumDeadline::get() + 10u32.into();

		T::Currency::make_free_balance_be(&buyer, amount * 10u32.into());

		#[extrinsic_call]
		create_escrow(RawOrigin::Signed(buyer.clone()), seller.clone(), amount, description, deadline);

		assert!(Escrows::<T>::iter().count() == 1);
	}

	#[benchmark]
	fn release_funds() {
		let buyer: T::AccountId = account("buyer", 0, 0);
		let seller: T::AccountId = account("seller", 0, 0);
		let amount = T::MinimumEscrowAmount::get();
		let description = vec![0u8; 10];
		let deadline = frame_system::Pallet::<T>::block_number() + T::MinimumDeadline::get() + 10u32.into();

		T::Currency::make_free_balance_be(&buyer, amount * 10u32.into());

		assert_ok!(Pallet::<T>::create_escrow(
			RawOrigin::Signed(buyer.clone()).into(),
			seller.clone(),
			amount,
			description,
			deadline,
		));

		let escrow_id = Escrows::<T>::iter().next().unwrap().0;

		#[extrinsic_call]
		release_funds(RawOrigin::Signed(buyer), escrow_id);

		let escrow = Escrows::<T>::get(escrow_id).unwrap();
		assert!(escrow.status == EscrowStatus::Released);
	}

	#[benchmark]
	fn refund() {
		let buyer: T::AccountId = account("buyer", 0, 0);
		let seller: T::AccountId = account("seller", 0, 0);
		let amount = T::MinimumEscrowAmount::get();
		let description = vec![0u8; 10];
		let deadline = frame_system::Pallet::<T>::block_number() + T::MinimumDeadline::get() + 10u32.into();

		T::Currency::make_free_balance_be(&buyer, amount * 10u32.into());

		assert_ok!(Pallet::<T>::create_escrow(
			RawOrigin::Signed(buyer.clone()).into(),
			seller.clone(),
			amount,
			description,
			deadline,
		));

		let escrow_id = Escrows::<T>::iter().next().unwrap().0;

		#[extrinsic_call]
		refund(RawOrigin::Signed(seller), escrow_id);

		let escrow = Escrows::<T>::get(escrow_id).unwrap();
		assert!(escrow.status == EscrowStatus::Refunded);
	}

	#[benchmark]
	fn claim_expired() {
		let buyer: T::AccountId = account("buyer", 0, 0);
		let seller: T::AccountId = account("seller", 0, 0);
		let amount = T::MinimumEscrowAmount::get();
		let description = vec![0u8; 10];
		let deadline = frame_system::Pallet::<T>::block_number() + T::MinimumDeadline::get() + 1u32.into();

		T::Currency::make_free_balance_be(&buyer, amount * 10u32.into());

		assert_ok!(Pallet::<T>::create_escrow(
			RawOrigin::Signed(buyer.clone()).into(),
			seller.clone(),
			amount,
			description,
			deadline,
		));

		let escrow_id = Escrows::<T>::iter().next().unwrap().0;

		// Move past deadline
		frame_system::Pallet::<T>::set_block_number(deadline + 1u32.into());

		#[extrinsic_call]
		claim_expired(RawOrigin::Signed(buyer), escrow_id);

		let escrow = Escrows::<T>::get(escrow_id).unwrap();
		assert!(escrow.status == EscrowStatus::Expired);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}