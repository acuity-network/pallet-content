#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! # Content Pallet
//!
//! A lightweight content registry pallet.
//!
//! This pallet tracks only control metadata for on-chain content items and emits
//! structured events for indexing and off-chain processing.
//! It is designed for use with `acuity-index` (`https://github.com/acuity-network/acuity-index`).
//!
//! The pallet provides the following capabilities:
//! - Publish a new item with an account- and namespace-derived `ItemId`.
//! - Publish item revisions.
//! - Retract an item.
//! - Disable revisioning or retraction permissions after creation.

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use polkadot_sdk::{frame_support, frame_system, sp_io};
use sp_io::hashing::blake2_256;

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

pub use pallet::*;

/// Flag bit allowing an item to accept new revisions.
pub const REVISIONABLE: u8 = 1 << 0;
/// Flag bit allowing an item to be retracted later.
pub const RETRACTABLE: u8 = 1 << 1;
/// Flag bit marking an item as retracted.
pub const RETRACTED: u8 = 1 << 2;
const ALLOWED_FLAGS: u8 = REVISIONABLE | RETRACTABLE;

/// Caller-supplied entropy used when deriving a deterministic [`ItemId`].
#[derive(PartialEq, Clone, Debug, TypeInfo, Encode, Decode, DecodeWithMemTracking, Default)]
pub struct Nonce([u8; 32]);

/// FRAME pallet implementation.
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Minimal on-chain state tracked for a published content item.
    #[derive(PartialEq, Clone, Debug, TypeInfo, Encode, Decode, MaxEncodedLen)]
    pub struct Item<AccountId> {
        /// Account that currently controls the item.
        pub owner: AccountId,
        /// Latest published revision number for the item.
        pub revision_id: RevisionId,
        /// Lifecycle and permission flags for the item.
        pub flags: u8,
    }

    /// Deterministic identifier for a content item.
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

    /// Fixed-width payload reference emitted with each revision.
    #[derive(PartialEq, Clone, Debug, Encode, Decode, TypeInfo, DecodeWithMemTracking, Default)]
    pub struct IpfsHash(pub [u8; 32]);

    /// Monotonic revision counter for an item.
    pub type RevisionId = u32;

    /// Configuration for the content pallet.
    #[pallet::config]
    pub trait Config: polkadot_sdk::frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        /// Weight implementation for this pallet's dispatchables.
        type WeightInfo: WeightInfo;
        /// Namespace value mixed into [`ItemId`] derivation to separate deployments.
        type ItemIdNamespace: Get<u32>;
        /// Maximum number of parents that can be attached during item creation.
        type MaxParents: Get<u32>;
        /// Maximum number of linked items that can be attached to a revision.
        type MaxLinks: Get<u32>;
        /// Maximum number of mentioned accounts that can be attached to a revision.
        type MaxMentions: Get<u32>;
    }

    /// Pallet type for content item lifecycle management.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Publishes a new item and its initial revision.
        ///
        /// The item id is derived from the signer, the supplied [`Nonce`], and
        /// [`Config::ItemIdNamespace`]. The call persists only ownership,
        /// revision, and flag metadata; graph edges and the payload reference are
        /// emitted in events for off-chain indexing.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::publish_item(
            parents.len() as u32,
            links.len() as u32,
            mentions.len() as u32,
        ))]
        pub fn publish_item(
            origin: OriginFor<T>,
            nonce: Nonce,
            parents: BoundedVec<ItemId, T::MaxParents>,
            flags: u8,
            links: BoundedVec<ItemId, T::MaxLinks>,
            mentions: BoundedVec<T::AccountId, T::MaxMentions>,
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
                mentions,
                ipfs_hash,
            });

            Ok(())
        }

        /// Publishes a new revision for an existing item.
        ///
        /// Only the current item owner can publish revisions, and only while the
        /// item is marked [`REVISIONABLE`] and not [`RETRACTED`].
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::publish_revision(
            links.len() as u32,
            mentions.len() as u32,
        ))]
        pub fn publish_revision(
            origin: OriginFor<T>,
            item_id: ItemId,
            links: BoundedVec<ItemId, T::MaxLinks>,
            mentions: BoundedVec<T::AccountId, T::MaxMentions>,
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
                mentions,
                ipfs_hash,
            });

            Ok(())
        }

        /// Marks an item as retracted.
        ///
        /// Only the owner can retract, and only while the item still has the
        /// [`RETRACTABLE`] permission bit set.
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

        /// Permanently disables future revisions for an item.
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

        /// Permanently disables future retraction for an item.
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
        /// Derives the deterministic identifier used for a newly published item.
        fn get_item_id(account: T::AccountId, nonce: Nonce) -> ItemId {
            let mut item_id = ItemId::default();
            item_id.0.copy_from_slice(&blake2_256(
                &[
                    account.encode(),
                    nonce.encode(),
                    T::ItemIdNamespace::get().encode(),
                ]
                .concat(),
            ));
            item_id
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new item was created.
        PublishItem {
            /// Deterministic identifier of the new item.
            item_id: ItemId,
            /// Account that now owns the item.
            owner: T::AccountId,
            /// Parent items declared at creation time.
            parents: BoundedVec<ItemId, T::MaxParents>,
            /// Initial lifecycle flags stored for the item.
            flags: u8,
        },
        /// A new revision was published for an item.
        PublishRevision {
            /// Identifier of the revised item.
            item_id: ItemId,
            /// Account that owns the revised item.
            owner: T::AccountId,
            /// Revision number that was just published.
            revision_id: RevisionId,
            /// Linked items declared by the revision.
            links: BoundedVec<ItemId, T::MaxLinks>,
            /// Mentioned accounts declared by the revision.
            mentions: BoundedVec<T::AccountId, T::MaxMentions>,
            /// Off-chain payload reference associated with the revision.
            ipfs_hash: IpfsHash,
        },
        /// An item was marked as retracted.
        RetractItem {
            /// Identifier of the retracted item.
            item_id: ItemId,
            /// Account that performed the retraction.
            owner: T::AccountId,
        },
        /// Revision publishing was permanently disabled for an item.
        SetNotRevsionable {
            /// Identifier of the affected item.
            item_id: ItemId,
            /// Account that changed the revision permission.
            owner: T::AccountId,
        },
        /// Retraction was permanently disabled for an item.
        SetNotRetractable {
            /// Identifier of the affected item.
            item_id: ItemId,
            /// Account that changed the retraction permission.
            owner: T::AccountId,
        },
    }

    /// Errors returned by the content pallet.
    #[pallet::error]
    pub enum Error<T> {
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

    /// Canonical item metadata keyed by deterministic [`ItemId`].
    #[pallet::storage]
    #[pallet::getter(fn item)]
    pub type ItemState<T: Config> =
        StorageMap<_, Blake2_128Concat, ItemId, Item<T::AccountId>, OptionQuery>;
}
