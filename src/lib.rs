// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::dispatch::DispatchResult;
use frame_system::ensure_signed;
use sp_io::hashing::blake2_256;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod tests;

/// Enable `dev_mode` for this pallet.
#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[derive(PartialEq, Clone, Debug, TypeInfo, Encode, Decode, Default)]
    pub struct Item<AccountId> {
        owner: Option<AccountId>, // Owner of the item. None is
        revision_id: u32,         // Latest revision_id
        retracted: bool,
    }

    #[derive(PartialEq, Clone, Debug, TypeInfo, Encode, Decode)]
    pub struct Nonce([u8; 32]);

    #[derive(PartialEq, Clone, Debug, TypeInfo, Default, Encode, Decode)]
    pub struct ItemId([u8; 32]);

    #[derive(PartialEq, Clone, Debug, Encode, Decode, TypeInfo)]
    pub struct IpfsHash([u8; 32]);

    #[pallet::config]
    pub trait Config: pallet_balances::Config + frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
    // method.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // No need to define a `call_index` attribute here because of `dev_mode`.
        // No need to define a `weight` attribute here because of `dev_mode`.
        pub fn publish_item(
            origin: OriginFor<T>,
            nonce: Nonce,
            ipfs_hash: IpfsHash,
        ) -> DispatchResult {
            let account = ensure_signed(origin)?;
            // Get item_id for the new item.
            let item_id = Self::get_item_id(account.clone(), nonce);
            // Ensure the item does not already exist.
            if <ItemState<T>>::contains_key(&item_id) {
                return Err(Error::<T>::ItemAlreadyExists.into());
            }
            // Store item in state.
            let item = Item {
                owner: Some(account.clone()),
                revision_id: 0,
                retracted: false,
            };
            <ItemState<T>>::insert(&item_id, item);
            // Emit event to log.
            Self::deposit_event(Event::PublishRevision {
                item_id,
                owner: account,
                revision_id: 0,
                ipfs_hash,
            });

            Ok(())
        }

        pub fn retract(origin: OriginFor<T>) -> DispatchResult {
            let account = ensure_signed(origin)?;
            Ok(())
        }

        pub fn publish_revision(
            origin: OriginFor<T>,
            item_id: ItemId,
            ipfs_hash: IpfsHash,
        ) -> DispatchResult {
            let account = ensure_signed(origin)?;

            let mut item = <ItemState<T>>::get(&item_id).ok_or(Error::<T>::ItemNotFound)?;

            if item.owner != Some(account.clone()) {
                return Err(Error::<T>::WrongAccount.into());
            }

            let revision_id = item.revision_id + 1;
            item.revision_id = revision_id;

            <ItemState<T>>::insert(&item_id, item);

            Self::deposit_event(Event::PublishRevision {
                item_id,
                owner: account,
                revision_id,
                ipfs_hash,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn get_item_id(account: T::AccountId, nonce: Nonce) -> ItemId {
            let mut item_id = ItemId::default();
            item_id
                .0
                .copy_from_slice(&blake2_256(&[account.encode(), nonce.encode()].concat()));
            item_id
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PublishRevision {
            item_id: ItemId,
            owner: T::AccountId,
            revision_id: u32,
            ipfs_hash: IpfsHash,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// The item could not be found.
        ItemAlreadyExists,
        /// The item could not be found.
        ItemNotFound,
        /// The sell order could not be found.
        WrongAccount,
    }

    #[pallet::storage]
    pub type ItemState<T: Config> = StorageMap<_, _, ItemId, Item<T::AccountId>>;
}
