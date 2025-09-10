#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame::pallet]
pub mod pallet {
	use super::WeightInfo;
	use frame::{
		deps::{
			frame_support::{
				pallet_prelude::*,
				traits::{Currency, ExistenceRequirement, ReservableCurrency},
				PalletId,
			},
			frame_system::pallet_prelude::*,
		},
		prelude::{Saturating, Hash},
	};
	use scale_info::prelude::vec::Vec;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame::deps::frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum EscrowStatus {
		Active,
		Released,
		Refunded,
		Expired,
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct EscrowDetails<T: Config> {
		pub buyer: T::AccountId,
		pub seller: T::AccountId,
		pub amount: BalanceOf<T>,
		pub description: BoundedVec<u8, T::MaxDescriptionLength>,
		pub deadline: BlockNumberFor<T>,
		pub status: EscrowStatus,
		pub created_at: BlockNumberFor<T>,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame::deps::frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame::deps::frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type MinimumEscrowAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MaximumEscrowAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MinimumDeadline: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type MaximumDeadline: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type MaxDescriptionLength: Get<u32>;

		#[pallet::constant]
		type MaxActiveEscrowsPerUser: Get<u32>;

		#[pallet::constant]
		type PlatformFeePercent: Get<u32>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn escrows)]
	pub type Escrows<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, EscrowDetails<T>>;
	
	impl<T: Config> Pallet<T> {
		pub fn iter_keys() -> impl Iterator<Item = T::Hash> {
			Escrows::<T>::iter_keys()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn next_escrow_id)]
	pub type NextEscrowId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn user_escrows)]
	pub type UserEscrows<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		bool, // true for buyer, false for seller
		BoundedVec<T::Hash, T::MaxActiveEscrowsPerUser>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn active_escrow_count)]
	pub type ActiveEscrowCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		EscrowCreated {
			escrow_id: T::Hash,
			buyer: T::AccountId,
			seller: T::AccountId,
			amount: BalanceOf<T>,
			deadline: BlockNumberFor<T>,
		},
		EscrowReleased {
			escrow_id: T::Hash,
			buyer: T::AccountId,
			seller: T::AccountId,
			amount: BalanceOf<T>,
			fee: BalanceOf<T>,
		},
		EscrowRefunded {
			escrow_id: T::Hash,
			buyer: T::AccountId,
			seller: T::AccountId,
			amount: BalanceOf<T>,
		},
		EscrowExpired {
			escrow_id: T::Hash,
			buyer: T::AccountId,
			amount: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidAmount,
		AmountTooSmall,
		AmountTooLarge,
		InvalidDeadline,
		DeadlineTooSoon,
		DeadlineTooFar,
		DescriptionTooLong,
		EscrowNotFound,
		NotAuthorized,
		EscrowAlreadyProcessed,
		EscrowNotExpired,
		TooManyActiveEscrows,
		SelfEscrowNotAllowed,
		InsufficientBalance,
		TransferFailed,
		Overflow,
		Underflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_escrow())]
		pub fn create_escrow(
			origin: OriginFor<T>,
			seller: T::AccountId,
			amount: BalanceOf<T>,
			description: Vec<u8>,
			deadline: BlockNumberFor<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			ensure!(buyer != seller, Error::<T>::SelfEscrowNotAllowed);

			ensure!(amount >= T::MinimumEscrowAmount::get(), Error::<T>::AmountTooSmall);
			ensure!(amount <= T::MaximumEscrowAmount::get(), Error::<T>::AmountTooLarge);

			ensure!(
				description.len() <= T::MaxDescriptionLength::get() as usize,
				Error::<T>::DescriptionTooLong
			);

			let current_block = frame::deps::frame_system::Pallet::<T>::block_number();
			ensure!(
				deadline > current_block + T::MinimumDeadline::get(),
				Error::<T>::DeadlineTooSoon
			);
			ensure!(
				deadline <= current_block + T::MaximumDeadline::get(),
				Error::<T>::DeadlineTooFar
			);

			let active_count = ActiveEscrowCount::<T>::get(&buyer);
			ensure!(
				active_count < T::MaxActiveEscrowsPerUser::get(),
				Error::<T>::TooManyActiveEscrows
			);

			T::Currency::reserve(&buyer, amount)?;

			let escrow_id_counter = NextEscrowId::<T>::get();
			let escrow_id = T::Hashing::hash_of(&(escrow_id_counter, &buyer, &seller, &amount));

			let bounded_description = BoundedVec::try_from(description)
				.map_err(|_| Error::<T>::DescriptionTooLong)?;

			let escrow = EscrowDetails {
				buyer: buyer.clone(),
				seller: seller.clone(),
				amount,
				description: bounded_description,
				deadline,
				status: EscrowStatus::Active,
				created_at: current_block,
			};

			Escrows::<T>::insert(&escrow_id, &escrow);
			NextEscrowId::<T>::put(escrow_id_counter.saturating_add(1));

			UserEscrows::<T>::mutate(&buyer, true, |escrows| {
				escrows.try_push(escrow_id).map_err(|_| Error::<T>::TooManyActiveEscrows)
			})?;

			UserEscrows::<T>::mutate(&seller, false, |escrows| {
				escrows.try_push(escrow_id).map_err(|_| Error::<T>::TooManyActiveEscrows)
			})?;

			ActiveEscrowCount::<T>::mutate(&buyer, |count| *count = count.saturating_add(1));

			Self::deposit_event(Event::EscrowCreated {
				escrow_id,
				buyer,
				seller,
				amount,
				deadline,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::release_funds())]
		pub fn release_funds(origin: OriginFor<T>, escrow_id: T::Hash) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;

			ensure!(escrow.buyer == caller, Error::<T>::NotAuthorized);
			ensure!(escrow.status == EscrowStatus::Active, Error::<T>::EscrowAlreadyProcessed);

			let fee_basis_points = T::PlatformFeePercent::get();
			let fee_amount = escrow.amount
				.saturating_mul(fee_basis_points.into())
				.checked_div(&10000u32.into())
				.ok_or(Error::<T>::Underflow)?;
			
			let amount_to_seller = escrow.amount.saturating_sub(fee_amount);

			T::Currency::unreserve(&escrow.buyer, escrow.amount);

			T::Currency::transfer(
				&escrow.buyer,
				&escrow.seller,
				amount_to_seller,
				ExistenceRequirement::AllowDeath,
			)?;

			escrow.status = EscrowStatus::Released;
			Escrows::<T>::insert(&escrow_id, &escrow);

			ActiveEscrowCount::<T>::mutate(&escrow.buyer, |count| *count = count.saturating_sub(1));

			Self::deposit_event(Event::EscrowReleased {
				escrow_id,
				buyer: escrow.buyer.clone(),
				seller: escrow.seller.clone(),
				amount: amount_to_seller,
				fee: fee_amount,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::refund())]
		pub fn refund(origin: OriginFor<T>, escrow_id: T::Hash) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;

			ensure!(escrow.seller == caller, Error::<T>::NotAuthorized);
			ensure!(escrow.status == EscrowStatus::Active, Error::<T>::EscrowAlreadyProcessed);

			T::Currency::unreserve(&escrow.buyer, escrow.amount);

			escrow.status = EscrowStatus::Refunded;
			Escrows::<T>::insert(&escrow_id, &escrow);

			ActiveEscrowCount::<T>::mutate(&escrow.buyer, |count| *count = count.saturating_sub(1));

			Self::deposit_event(Event::EscrowRefunded {
				escrow_id,
				buyer: escrow.buyer.clone(),
				seller: escrow.seller.clone(),
				amount: escrow.amount,
			});

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::claim_expired())]
		pub fn claim_expired(origin: OriginFor<T>, escrow_id: T::Hash) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;

			ensure!(escrow.buyer == caller, Error::<T>::NotAuthorized);
			ensure!(escrow.status == EscrowStatus::Active, Error::<T>::EscrowAlreadyProcessed);

			let current_block = frame::deps::frame_system::Pallet::<T>::block_number();
			ensure!(current_block > escrow.deadline, Error::<T>::EscrowNotExpired);

			T::Currency::unreserve(&escrow.buyer, escrow.amount);

			escrow.status = EscrowStatus::Expired;
			Escrows::<T>::insert(&escrow_id, &escrow);

			ActiveEscrowCount::<T>::mutate(&escrow.buyer, |count| *count = count.saturating_sub(1));

			Self::deposit_event(Event::EscrowExpired {
				escrow_id,
				buyer: escrow.buyer.clone(),
				amount: escrow.amount,
			});

			Ok(())
		}
	}
}