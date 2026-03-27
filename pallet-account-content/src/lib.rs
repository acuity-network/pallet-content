#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! # Account Content Pallet
//!
//! A lightweight account-scoped index of `pallet-content` item ids.
//!
//! This pallet ports the `AcuityAccountItems` Solidity contract into FRAME. Each
//! account can maintain an ordered list of content `ItemId`s that it owns in
//! `pallet-content`, while supporting O(1) membership checks and removals.

pub use pallet::*;
use polkadot_sdk::{frame_support, frame_system};

/// Benchmark definitions for the pallet.
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Weight traits and generated weight implementations.
pub mod weights;
pub use weights::*;

/// FRAME pallet implementation.
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_content::{ItemId, RETRACTED};

    /// Configuration for the account-content pallet.
    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: polkadot_sdk::frame_system::Config + pallet_content::Config {
        /// Aggregated runtime event type.
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;
        /// Weight implementation for this pallet's dispatchables.
        type WeightInfo: WeightInfo;
        /// Maximum number of content items one account can index locally.
        type MaxItemsPerAccount: Get<u32>;
    }

    /// Pallet type for account-scoped content lists.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Adds a content item to the caller's ordered list.
        ///
        /// The referenced item must exist in `pallet-content`, must not be
        /// retracted, and must currently be owned by the caller.
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

        /// Removes a content item from the caller's ordered list.
        ///
        /// Removal uses swap-with-last semantics so membership checks and
        /// deletions stay O(1).
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
        /// Ensures the referenced item exists, is not retracted, and belongs to the account.
        fn ensure_item_owned_by(account: &T::AccountId, item_id: &ItemId) -> Result<(), Error<T>> {
            let item =
                pallet_content::ItemState::<T>::get(item_id).ok_or(Error::<T>::ItemNotFound)?;
            ensure!(item.flags & RETRACTED == 0, Error::<T>::ItemRetracted);
            ensure!(item.owner == *account, Error::<T>::WrongAccount);
            Ok(())
        }

        /// Returns whether the supplied account has already added the item.
        pub fn get_item_exists(account: T::AccountId, item_id: ItemId) -> bool {
            Self::get_item_exists_by_account(account, item_id)
        }

        /// Returns the number of items currently indexed by the supplied account.
        pub fn get_item_count(account: T::AccountId) -> u32 {
            Self::get_item_count_by_account(account)
        }

        /// Returns all item ids currently indexed by the supplied account.
        pub fn get_all_items(account: T::AccountId) -> BoundedVec<ItemId, T::MaxItemsPerAccount> {
            Self::get_all_items_by_account(account)
        }

        /// Returns whether the supplied account has already added the item.
        pub fn get_item_exists_by_account(account: T::AccountId, item_id: ItemId) -> bool {
            AccountItemIdIndex::<T>::get(account, item_id) > 0
        }

        /// Returns the current number of stored item ids for the supplied account.
        pub fn get_item_count_by_account(account: T::AccountId) -> u32 {
            u32::try_from(AccountItemIds::<T>::get(account).len()).unwrap_or(u32::MAX)
        }

        /// Returns the caller-facing ordered item list for the supplied account.
        pub fn get_all_items_by_account(
            account: T::AccountId,
        ) -> BoundedVec<ItemId, T::MaxItemsPerAccount> {
            AccountItemIds::<T>::get(account)
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An item was added to an account list.
        AddItem {
            /// Account that owns the list.
            account: T::AccountId,
            /// Item inserted into the account list.
            item_id: ItemId,
        },
        /// An item was removed from an account list.
        RemoveItem {
            /// Account that owns the list.
            account: T::AccountId,
            /// Item removed from the account list.
            item_id: ItemId,
        },
    }

    /// Errors returned by the account-content pallet.
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

    /// Ordered content item ids keyed by account.
    #[pallet::storage]
    #[pallet::getter(fn account_item_ids)]
    pub type AccountItemIds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<ItemId, T::MaxItemsPerAccount>,
        ValueQuery,
    >;

    /// Reverse lookup from `(account, item_id)` to `index + 1` in [`AccountItemIds`].
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
