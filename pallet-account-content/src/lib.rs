#![cfg_attr(not(feature = "std"), no_std)]

//! # Account Content Pallet
//!
//! A lightweight account-scoped index of `pallet-content` item ids.
//!
//! This pallet ports the `AcuityAccountItems` Solidity contract into FRAME. Each
//! account can maintain an ordered list of content `ItemId`s that it owns in
//! `pallet-content`, while supporting O(1) membership checks and removals.

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
        type MaxItemsPerAccount: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::add_item())]
        pub fn add_item(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult {
            let account = ensure_signed(origin)?;

            ensure!(
                AccountItemIdIndex::<T>::get(&account, &item_id) == 0,
                Error::<T>::ItemAlreadyAdded
            );

            Self::ensure_item_owned_by(&account, &item_id)?;

            AccountItemIds::<T>::try_mutate(&account, |item_ids| -> DispatchResult {
                item_ids
                    .try_push(item_id.clone())
                    .map_err(|_| Error::<T>::AccountItemsFull)?;
                let index =
                    u32::try_from(item_ids.len()).map_err(|_| Error::<T>::AccountItemsFull)?;
                AccountItemIdIndex::<T>::insert(&account, &item_id, index);
                Ok(())
            })?;

            Self::deposit_event(Event::AddItem { account, item_id });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_item())]
        pub fn remove_item(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult {
            let account = ensure_signed(origin)?;
            let index_plus_one = AccountItemIdIndex::<T>::get(&account, &item_id);

            ensure!(index_plus_one > 0, Error::<T>::ItemNotAdded);

            Self::ensure_item_owned_by(&account, &item_id)?;

            let mut item_ids = AccountItemIds::<T>::get(&account);
            let last_index = item_ids
                .len()
                .checked_sub(1)
                .ok_or(Error::<T>::ItemNotAdded)?;
            let item_index =
                usize::try_from(index_plus_one - 1).map_err(|_| Error::<T>::IndexOverflow)?;

            AccountItemIdIndex::<T>::remove(&account, &item_id);

            if item_index != last_index {
                let moving_item_id = item_ids[last_index].clone();
                item_ids[item_index] = moving_item_id.clone();
                AccountItemIdIndex::<T>::insert(&account, &moving_item_id, index_plus_one);
            }

            item_ids.pop();
            AccountItemIds::<T>::insert(&account, item_ids);

            Self::deposit_event(Event::RemoveItem { account, item_id });

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

        pub fn get_item_exists(account: T::AccountId, item_id: ItemId) -> bool {
            Self::get_item_exists_by_account(account, item_id)
        }

        pub fn get_item_count(account: T::AccountId) -> u32 {
            Self::get_item_count_by_account(account)
        }

        pub fn get_all_items(account: T::AccountId) -> BoundedVec<ItemId, T::MaxItemsPerAccount> {
            Self::get_all_items_by_account(account)
        }

        pub fn get_item_exists_by_account(account: T::AccountId, item_id: ItemId) -> bool {
            AccountItemIdIndex::<T>::get(account, item_id) > 0
        }

        pub fn get_item_count_by_account(account: T::AccountId) -> u32 {
            u32::try_from(AccountItemIds::<T>::get(account).len()).unwrap_or(u32::MAX)
        }

        pub fn get_all_items_by_account(
            account: T::AccountId,
        ) -> BoundedVec<ItemId, T::MaxItemsPerAccount> {
            AccountItemIds::<T>::get(account)
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AddItem {
            account: T::AccountId,
            item_id: ItemId,
        },
        RemoveItem {
            account: T::AccountId,
            item_id: ItemId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The item is already in the account list.
        ItemAlreadyAdded,
        /// The item is not in the account list.
        ItemNotAdded,
        /// The referenced content item could not be found.
        ItemNotFound,
        /// The referenced content item has been retracted.
        ItemRetracted,
        /// The signer does not own the referenced content item.
        WrongAccount,
        /// The account has reached the maximum supported number of items.
        AccountItemsFull,
        /// A stored index could not be converted on this platform.
        IndexOverflow,
    }

    #[pallet::storage]
    #[pallet::getter(fn account_item_ids)]
    pub type AccountItemIds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<ItemId, T::MaxItemsPerAccount>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn account_item_id_index)]
    pub type AccountItemIdIndex<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        ItemId,
        u32,
        ValueQuery,
    >;
}
