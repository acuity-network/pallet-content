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
        pub fn publish_revsion(
            origin: OriginFor<T>,
            nonce: Nonce,
            ipfs_hash: IpfsHash,
        ) -> DispatchResult {
            let account = ensure_signed(origin)?;

            let item_id = Self::get_item_id(account.clone(), nonce);
            let revision_id = <RevisionCount<T>>::get(&item_id);
            <RevisionCount<T>>::insert(&item_id, revision_id + 1);

            Self::deposit_event(Event::PublishRevision {
                account,
                item_id,
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
            account: T::AccountId,
            item_id: ItemId,
            revision_id: u32,
            ipfs_hash: IpfsHash,
        },
    }

    #[pallet::storage]
    pub type RevisionCount<T: Config> = StorageMap<_, _, ItemId, u32, ValueQuery>;
}
