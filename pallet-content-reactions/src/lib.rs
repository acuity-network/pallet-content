#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! # Content Reactions Pallet
//!
//! A lightweight account-scoped emoji reaction pallet for `pallet-content`.
//!
//! Each account can associate a bounded set of Unicode scalar values with any
//! content `ItemId` revision. Reactions are ephemeral: the pallet stores no
//! on-chain state and instead emits a `SetReactions` event containing the full
//! reaction set for every `(item_id, revision_id, account)` tuple.

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
    use pallet_content::{Item, ItemId, RevisionId, RETRACTED};

    /// Unicode scalar value stored as a reaction.
    #[derive(
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Encode,
        Decode,
        DecodeWithMemTracking,
        TypeInfo,
        MaxEncodedLen,
        Debug,
        Default,
    )]
    pub struct Emoji(pub u32);

    /// Reaction set for a single `(item_id, revision_id, account)` tuple.
    pub type ReactionsOf<T> = BoundedVec<Emoji, <T as Config>::MaxEmojis>;

    /// Configuration for the content-reactions pallet.
    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: polkadot_sdk::frame_system::Config + pallet_content::Config {
        /// Aggregated runtime event type.
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;
        /// Weight implementation for this pallet's dispatchables.
        type WeightInfo: WeightInfo;
        /// Maximum number of distinct emoji reactions one account can attach to a revision.
        type MaxEmojis: Get<u32>;
    }

    /// Pallet type for per-account content reactions.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sets the caller's full reaction set for a specific item revision,
        /// replacing any prior reactions.
        ///
        /// The entire reaction set is emitted in a single `SetReactions` event.
        /// Duplicates within the provided set are rejected.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::set_reactions())]
        pub fn set_reactions(
            origin: OriginFor<T>,
            item_id: ItemId,
            revision_id: RevisionId,
            reactions: ReactionsOf<T>,
        ) -> DispatchResult {
            let reactor = ensure_signed(origin)?;

            Self::ensure_no_duplicate_emojis(&reactions)?;
            for emoji in reactions.iter() {
                Self::ensure_valid_emoji(*emoji)?;
            }
            let item_owner = Self::get_item_and_validate_revision(&item_id, revision_id)?.owner;

            Self::deposit_event(Event::SetReactions {
                item_id,
                revision_id,
                item_owner,
                reactor,
                reactions,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Validates that the provided value is a non-zero Unicode scalar value.
        fn ensure_valid_emoji(emoji: Emoji) -> Result<(), Error<T>> {
            ensure!(emoji.0 != 0, Error::<T>::InvalidEmoji);
            ensure!(char::from_u32(emoji.0).is_some(), Error::<T>::InvalidEmoji);
            Ok(())
        }

        /// Validates that the provided reaction set contains no duplicate emojis.
        fn ensure_no_duplicate_emojis(reactions: &ReactionsOf<T>) -> Result<(), Error<T>> {
            for (i, a) in reactions.iter().enumerate() {
                for b in reactions.iter().skip(i + 1) {
                    ensure!(a != b, Error::<T>::DuplicateEmoji);
                }
            }
            Ok(())
        }

        /// Loads an item and ensures the requested revision exists and is still active.
        fn get_item_and_validate_revision(
            item_id: &ItemId,
            revision_id: RevisionId,
        ) -> Result<Item<T::AccountId>, Error<T>> {
            let item =
                pallet_content::ItemState::<T>::get(item_id).ok_or(Error::<T>::ItemNotFound)?;
            ensure!(item.flags & RETRACTED == 0, Error::<T>::ItemRetracted);
            ensure!(
                revision_id <= item.revision_id,
                Error::<T>::RevisionNotFound
            );
            Ok(item)
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The caller's reaction set for a specific item revision was set.
        SetReactions {
            /// Reacted content item.
            item_id: ItemId,
            /// Revision that received the reactions.
            revision_id: RevisionId,
            /// Owner of the reacted item.
            item_owner: T::AccountId,
            /// Account that set the reactions.
            reactor: T::AccountId,
            /// The full set of reactions.
            reactions: ReactionsOf<T>,
        },
    }

    /// Errors returned by the content-reactions pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// The referenced content item could not be found.
        ItemNotFound,
        /// The referenced content item has been retracted.
        ItemRetracted,
        /// The referenced revision could not be found for the item.
        RevisionNotFound,
        /// The provided emoji value is not a valid non-zero Unicode scalar value.
        InvalidEmoji,
        /// The provided reaction set contains duplicate emojis.
        DuplicateEmoji,
    }
}
