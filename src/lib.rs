#![cfg_attr(not(feature = "std"), no_std)]

//! # Content Pallet
//!
//! A lightweight content registry pallet.
//!
//! This pallet tracks only control metadata for on-chain content items and emits
//! structured events for indexing and off-chain processing.
//! It is designed for use with `acuity-index` (`https://github.com/acuity-network/acuity-index`).
//!
//! The pallet provides the following capabilities:
//! - Publish a new item with an account-derived `ItemId`.
//! - Publish item revisions.
//! - Retract an item.
//! - Disable revisioning or retraction permissions after creation.

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use polkadot_sdk::{frame_support, frame_system, sp_io};
use sp_io::hashing::blake2_256;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

pub use pallet::*;

const REVISIONABLE: u8 = 1 << 0;
const RETRACTABLE: u8 = 1 << 1;
const RETRACTED: u8 = 1 << 2;
const ALLOWED_FLAGS: u8 = REVISIONABLE | RETRACTABLE;

#[derive(PartialEq, Clone, Debug, TypeInfo, Encode, Decode, DecodeWithMemTracking, Default)]
pub struct Nonce([u8; 32]);

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[derive(PartialEq, Clone, Debug, TypeInfo, Encode, Decode, MaxEncodedLen)]
    pub struct Item<AccountId> {
        pub owner: AccountId, // Owner of the item
        pub revision_id: u32, // Latest revision_id
        pub flags: u8,
    }

    #[derive(
        PartialEq,
        Clone,
        Debug,
        TypeInfo,
        Default,
        Encode,
        Decode,
        DecodeWithMemTracking,
        MaxEncodedLen,
    )]
    pub struct ItemId(pub [u8; 32]);

    #[derive(PartialEq, Clone, Debug, Encode, Decode, TypeInfo, DecodeWithMemTracking, Default)]
    pub struct IpfsHash(pub [u8; 32]);

    #[pallet::config]
    pub trait Config: polkadot_sdk::frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        type WeightInfo: WeightInfo;
        type MaxParents: Get<u32>;
        type MaxLinks: Get<u32>;
    }

    // Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
    // method.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::publish_item(parents.len() as u32, links.len() as u32))]
        pub fn publish_item(
            origin: OriginFor<T>,
            nonce: Nonce,
            parents: BoundedVec<ItemId, T::MaxParents>,
            flags: u8,
            links: BoundedVec<ItemId, T::MaxLinks>,
            ipfs_hash: IpfsHash,
        ) -> DispatchResult {
            let account = ensure_signed(origin)?;

            if flags & !ALLOWED_FLAGS != 0 {
                return Err(Error::<T>::InvalidFlags.into());
            }

            // Get item_id for the new item.
            let item_id = Self::get_item_id(account.clone(), nonce);
            // Ensure the item does not already exist.
            if <ItemState<T>>::contains_key(&item_id) {
                return Err(Error::<T>::ItemAlreadyExists.into());
            }
            // Store item in state.
            let item = Item {
                owner: account.clone(),
                revision_id: 0,
                flags,
            };
            <ItemState<T>>::insert(&item_id, item);
            // Emit event to log.
            Self::deposit_event(Event::PublishItem {
                item_id: item_id.clone(),
                owner: account.clone(),
                parents,
                flags,
            });
            Self::deposit_event(Event::PublishRevision {
                item_id: item_id.clone(),
                owner: account,
                revision_id: 0,
                links,
                ipfs_hash,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::publish_revision(links.len() as u32))]
        pub fn publish_revision(
            origin: OriginFor<T>,
            item_id: ItemId,
            links: BoundedVec<ItemId, T::MaxLinks>,
            ipfs_hash: IpfsHash,
        ) -> DispatchResult {
            let account = ensure_signed(origin)?;

            let mut item = <ItemState<T>>::get(&item_id).ok_or(Error::<T>::ItemNotFound)?;

            if item.owner != account.clone() {
                return Err(Error::<T>::WrongAccount.into());
            }

            if item.flags & RETRACTED != 0 {
                return Err(Error::<T>::ItemRetracted.into());
            }

            if item.flags & REVISIONABLE == 0 {
                return Err(Error::<T>::ItemNotRevisionable.into());
            }

            let revision_id = item
                .revision_id
                .checked_add(1)
                .ok_or(Error::<T>::RevisionIdOverflow)?;
            item.revision_id = revision_id;

            <ItemState<T>>::insert(&item_id, item);

            Self::deposit_event(Event::PublishRevision {
                item_id,
                owner: account,
                revision_id,
                links,
                ipfs_hash,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::retract_item())]
        pub fn retract_item(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult {
            let account = ensure_signed(origin)?;
            let mut item = <ItemState<T>>::get(&item_id).ok_or(Error::<T>::ItemNotFound)?;

            if item.owner != account.clone() {
                return Err(Error::<T>::WrongAccount.into());
            }

            if item.flags & RETRACTED != 0 {
                return Err(Error::<T>::ItemRetracted.into());
            }

            if item.flags & RETRACTABLE == 0 {
                return Err(Error::<T>::ItemNotRetractable.into());
            }

            item.flags = RETRACTED;
            <ItemState<T>>::insert(&item_id, item);
            Self::deposit_event(Event::RetractItem {
                item_id,
                owner: account,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::set_not_revisionable())]
        pub fn set_not_revisionable(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult {
            let account = ensure_signed(origin)?;
            let mut item = <ItemState<T>>::get(&item_id).ok_or(Error::<T>::ItemNotFound)?;

            if item.owner != account.clone() {
                return Err(Error::<T>::WrongAccount.into());
            }

            if item.flags & REVISIONABLE == 0 {
                return Err(Error::<T>::ItemNotRevisionable.into());
            }

            item.flags &= !REVISIONABLE;
            <ItemState<T>>::insert(&item_id, item);
            Self::deposit_event(Event::SetNotRevsionable {
                item_id,
                owner: account,
            });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_not_retractable())]
        pub fn set_not_retractable(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult {
            let account = ensure_signed(origin)?;
            let mut item = <ItemState<T>>::get(&item_id).ok_or(Error::<T>::ItemNotFound)?;

            if item.owner != account.clone() {
                return Err(Error::<T>::WrongAccount.into());
            }

            if item.flags & RETRACTABLE == 0 {
                return Err(Error::<T>::ItemNotRetractable.into());
            }

            item.flags &= !RETRACTABLE;
            <ItemState<T>>::insert(&item_id, item);
            Self::deposit_event(Event::SetNotRetractable {
                item_id,
                owner: account,
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
        // Success,
        PublishItem {
            item_id: ItemId,
            owner: T::AccountId,
            parents: BoundedVec<ItemId, T::MaxParents>,
            flags: u8,
        },
        PublishRevision {
            item_id: ItemId,
            owner: T::AccountId,
            revision_id: u32,
            links: BoundedVec<ItemId, T::MaxLinks>,
            ipfs_hash: IpfsHash,
        },
        RetractItem {
            item_id: ItemId,
            owner: T::AccountId,
        },
        SetNotRevsionable {
            item_id: ItemId,
            owner: T::AccountId,
        },
        SetNotRetractable {
            item_id: ItemId,
            owner: T::AccountId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        // Error,
        /// The item already exists.
        ItemAlreadyExists,
        /// The item could not be found.
        ItemNotFound,
        /// The item has been retracted.
        ItemRetracted,
        /// The item is not revisionable.
        ItemNotRevisionable,
        /// The item is not retractable.
        ItemNotRetractable,
        /// Wrong account.
        WrongAccount,
        /// Flags contain unsupported bits.
        InvalidFlags,
        /// Revision id overflowed.
        RevisionIdOverflow,
    }

    #[pallet::storage]
    #[pallet::getter(fn item)]
    pub type ItemState<T: Config> =
        StorageMap<_, Blake2_128Concat, ItemId, Item<T::AccountId>, OptionQuery>;
}
