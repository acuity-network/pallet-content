#![cfg_attr(not(feature = "std"), no_std)]

//! # Account Profile Pallet
//!
//! A lightweight account-scoped profile pointer into `pallet-content`.
//!
//! This pallet ports the `AcuityAccountProfile` Solidity contract into FRAME.
//! Each account can associate itself with a single `pallet-content` `ItemId`
//! that it currently owns.

pub use pallet::*;
use polkadot_sdk::{frame_support, frame_system};

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_content::{ItemId, RETRACTED};

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: polkadot_sdk::frame_system::Config + pallet_content::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::set_profile())]
        pub fn set_profile(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult {
            let account = ensure_signed(origin)?;

            Self::ensure_item_owned_by(&account, &item_id)?;

            AccountProfile::<T>::insert(&account, item_id.clone());

            Self::deposit_event(Event::ProfileSet { account, item_id });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_item_owned_by(account: &T::AccountId, item_id: &ItemId) -> Result<(), Error<T>> {
            let item =
                pallet_content::ItemState::<T>::get(item_id).ok_or(Error::<T>::ItemNotFound)?;
            ensure!(item.flags & RETRACTED == 0, Error::<T>::ItemRetracted);
            ensure!(item.owner == *account, Error::<T>::WrongAccount);
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProfileSet {
            account: T::AccountId,
            item_id: ItemId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The referenced content item could not be found.
        ItemNotFound,
        /// The referenced content item has been retracted.
        ItemRetracted,
        /// The signer does not own the referenced content item.
        WrongAccount,
    }

    #[pallet::storage]
    #[pallet::getter(fn account_profile)]
    pub type AccountProfile<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ItemId, OptionQuery>;
}
