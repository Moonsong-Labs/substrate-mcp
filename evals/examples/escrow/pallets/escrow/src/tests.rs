use crate::{mock::*, Error, Event, EscrowStatus};
use frame::deps::frame_support::{assert_err, assert_ok};
use frame::prelude::fungible::Inspect;

#[test]
fn create_escrow_works() {
	new_test_ext().execute_with(|| {
		let amount = 1000u128;
		let description = b"Test escrow".to_vec();
		let deadline = 100;
		
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			amount,
			description.clone(),
			deadline
		));

		System::assert_last_event(
			Event::EscrowCreated {
				escrow_id: Escrow::iter_keys().next().unwrap(),
				buyer: ALICE,
				seller: BOB,
				amount,
				deadline,
			}
			.into(),
		);

		assert_eq!(Balances::reserved_balance(&ALICE), amount);
		assert_eq!(Escrow::active_escrow_count(ALICE), 1);
	});
}

#[test]
fn create_escrow_fails_with_self_escrow() {
	new_test_ext().execute_with(|| {
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				ALICE,
				1000,
				b"Test".to_vec(),
				100
			),
			Error::<Test>::SelfEscrowNotAllowed
		);
	});
}

#[test]
fn create_escrow_fails_with_amount_too_small() {
	new_test_ext().execute_with(|| {
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				5, // Below minimum
				b"Test".to_vec(),
				100
			),
			Error::<Test>::AmountTooSmall
		);
	});
}

#[test]
fn create_escrow_fails_with_amount_too_large() {
	new_test_ext().execute_with(|| {
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				2_000_000, // Above maximum
				b"Test".to_vec(),
				100
			),
			Error::<Test>::AmountTooLarge
		);
	});
}

#[test]
fn create_escrow_fails_with_description_too_long() {
	new_test_ext().execute_with(|| {
		let description = vec![0u8; 501]; // Over 500 character limit
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				1000,
				description,
				100
			),
			Error::<Test>::DescriptionTooLong
		);
	});
}

#[test]
fn create_escrow_fails_with_deadline_too_soon() {
	new_test_ext().execute_with(|| {
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				1000,
				b"Test".to_vec(),
				5 // Less than minimum deadline
			),
			Error::<Test>::DeadlineTooSoon
		);
	});
}

#[test]
fn create_escrow_fails_with_deadline_too_far() {
	new_test_ext().execute_with(|| {
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				1000,
				b"Test".to_vec(),
				200_000 // More than maximum deadline
			),
			Error::<Test>::DeadlineTooFar
		);
	});
}

#[test]
fn release_funds_works() {
	new_test_ext().execute_with(|| {
		let amount = 1000u128;
		let description = b"Test escrow".to_vec();
		let deadline = 100;
		
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			amount,
			description.clone(),
			deadline
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		let bob_initial_balance = <Balances as Inspect<u64>>::balance(&BOB);
		
		assert_ok!(Escrow::release_funds(RuntimeOrigin::signed(ALICE), escrow_id));

		let fee = amount * 100 / 10000; // 1% fee
		let amount_to_seller = amount - fee;
		
		System::assert_last_event(
			Event::EscrowReleased {
				escrow_id,
				buyer: ALICE,
				seller: BOB,
				amount: amount_to_seller,
				fee,
			}
			.into(),
		);

		assert_eq!(Balances::reserved_balance(&ALICE), 0);
		assert_eq!(<Balances as Inspect<u64>>::balance(&BOB), bob_initial_balance + amount_to_seller);
		assert_eq!(Escrow::active_escrow_count(ALICE), 0);
		
		let escrow = Escrow::escrows(escrow_id).unwrap();
		assert_eq!(escrow.status, EscrowStatus::Released);
	});
}

#[test]
fn release_funds_fails_when_not_buyer() {
	new_test_ext().execute_with(|| {
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			1000,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		assert_err!(
			Escrow::release_funds(RuntimeOrigin::signed(BOB), escrow_id),
			Error::<Test>::NotAuthorized
		);
		
		assert_err!(
			Escrow::release_funds(RuntimeOrigin::signed(CHARLIE), escrow_id),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn release_funds_fails_when_already_processed() {
	new_test_ext().execute_with(|| {
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			1000,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		assert_ok!(Escrow::release_funds(RuntimeOrigin::signed(ALICE), escrow_id));
		
		assert_err!(
			Escrow::release_funds(RuntimeOrigin::signed(ALICE), escrow_id),
			Error::<Test>::EscrowAlreadyProcessed
		);
	});
}

#[test]
fn refund_works() {
	new_test_ext().execute_with(|| {
		let amount = 1000u128;
		let alice_initial_balance = <Balances as Inspect<u64>>::balance(&ALICE);
		
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			amount,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		assert_ok!(Escrow::refund(RuntimeOrigin::signed(BOB), escrow_id));
		
		System::assert_last_event(
			Event::EscrowRefunded {
				escrow_id,
				buyer: ALICE,
				seller: BOB,
				amount,
			}
			.into(),
		);

		assert_eq!(Balances::reserved_balance(&ALICE), 0);
		assert_eq!(<Balances as Inspect<u64>>::balance(&ALICE), alice_initial_balance);
		assert_eq!(Escrow::active_escrow_count(ALICE), 0);
		
		let escrow = Escrow::escrows(escrow_id).unwrap();
		assert_eq!(escrow.status, EscrowStatus::Refunded);
	});
}

#[test]
fn refund_fails_when_not_seller() {
	new_test_ext().execute_with(|| {
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			1000,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		assert_err!(
			Escrow::refund(RuntimeOrigin::signed(ALICE), escrow_id),
			Error::<Test>::NotAuthorized
		);
		
		assert_err!(
			Escrow::refund(RuntimeOrigin::signed(CHARLIE), escrow_id),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn refund_fails_when_already_processed() {
	new_test_ext().execute_with(|| {
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			1000,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		assert_ok!(Escrow::refund(RuntimeOrigin::signed(BOB), escrow_id));
		
		assert_err!(
			Escrow::refund(RuntimeOrigin::signed(BOB), escrow_id),
			Error::<Test>::EscrowAlreadyProcessed
		);
	});
}

#[test]
fn claim_expired_works() {
	new_test_ext().execute_with(|| {
		let amount = 1000u128;
		let alice_initial_balance = <Balances as Inspect<u64>>::balance(&ALICE);
		let deadline = 100;
		
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			amount,
			b"Test".to_vec(),
			deadline
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		// Move past deadline
		run_to_block(deadline + 1);
		
		assert_ok!(Escrow::claim_expired(RuntimeOrigin::signed(ALICE), escrow_id));
		
		System::assert_last_event(
			Event::EscrowExpired {
				escrow_id,
				buyer: ALICE,
				amount,
			}
			.into(),
		);

		assert_eq!(Balances::reserved_balance(&ALICE), 0);
		assert_eq!(<Balances as Inspect<u64>>::balance(&ALICE), alice_initial_balance);
		assert_eq!(Escrow::active_escrow_count(ALICE), 0);
		
		let escrow = Escrow::escrows(escrow_id).unwrap();
		assert_eq!(escrow.status, EscrowStatus::Expired);
	});
}

#[test]
fn claim_expired_fails_when_not_buyer() {
	new_test_ext().execute_with(|| {
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			1000,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		run_to_block(101);
		
		assert_err!(
			Escrow::claim_expired(RuntimeOrigin::signed(BOB), escrow_id),
			Error::<Test>::NotAuthorized
		);
		
		assert_err!(
			Escrow::claim_expired(RuntimeOrigin::signed(CHARLIE), escrow_id),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn claim_expired_fails_when_not_expired() {
	new_test_ext().execute_with(|| {
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			1000,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		assert_err!(
			Escrow::claim_expired(RuntimeOrigin::signed(ALICE), escrow_id),
			Error::<Test>::EscrowNotExpired
		);
	});
}

#[test]
fn claim_expired_fails_when_already_processed() {
	new_test_ext().execute_with(|| {
		assert_ok!(Escrow::create_escrow(
			RuntimeOrigin::signed(ALICE),
			BOB,
			1000,
			b"Test".to_vec(),
			100
		));

		let escrow_id = Escrow::iter_keys().next().unwrap();
		
		run_to_block(101);
		
		assert_ok!(Escrow::claim_expired(RuntimeOrigin::signed(ALICE), escrow_id));
		
		assert_err!(
			Escrow::claim_expired(RuntimeOrigin::signed(ALICE), escrow_id),
			Error::<Test>::EscrowAlreadyProcessed
		);
	});
}

#[test]
fn multiple_escrows_tracking_works() {
	new_test_ext().execute_with(|| {
		// Create multiple escrows
		for i in 0..5 {
			assert_ok!(Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				100 + (i as u128) * 10,
				format!("Escrow {}", i).into_bytes(),
				100 + (i as u64) * 10
			));
		}
		
		assert_eq!(Escrow::active_escrow_count(ALICE), 5);
		assert_eq!(Escrow::user_escrows(ALICE, true).len(), 5);
		assert_eq!(Escrow::user_escrows(BOB, false).len(), 5);
		
		// Release one escrow
		let escrow_id = Escrow::user_escrows(ALICE, true)[0];
		assert_ok!(Escrow::release_funds(RuntimeOrigin::signed(ALICE), escrow_id));
		
		assert_eq!(Escrow::active_escrow_count(ALICE), 4);
	});
}

#[test]
fn too_many_active_escrows_fails() {
	new_test_ext().execute_with(|| {
		// Create maximum allowed escrows
		for i in 0..100 {
			assert_ok!(Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				100 + i as u128,
				format!("Escrow {}", i).into_bytes(),
				100 + i as u64
			));
		}
		
		// Try to create one more
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				1000,
				b"One too many".to_vec(),
				200
			),
			Error::<Test>::TooManyActiveEscrows
		);
	});
}

#[test]
fn insufficient_balance_fails() {
	new_test_ext().execute_with(|| {
		assert_err!(
			Escrow::create_escrow(
				RuntimeOrigin::signed(ALICE),
				BOB,
				2_000_000, // More than Alice has
				b"Test".to_vec(),
				100
			),
			Error::<Test>::AmountTooLarge
		);
	});
}