#![cfg_attr(not(feature = "std"), no_std)]

//! # Content Reactions Pallet
//!
//! A lightweight account-scoped emoji reaction pallet for `pallet-content`.
//!
//! Each account can associate a bounded set of Unicode scalar values with any
//! content `ItemId` revision. Reactions are stored per
//! `(item_id, revision_id, account)`.

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
    use pallet_content::{Item, ItemId, RevisionId, RETRACTED};

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

    pub type ReactionsOf<T> = BoundedVec<Emoji, <T as Config>::MaxEmojis>;

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: polkadot_sdk::frame_system::Config + pallet_content::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        type MaxEmojis: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::add_reaction())]
        pub fn add_reaction(
            origin: OriginFor<T>,
            item_id: ItemId,
            revision_id: RevisionId,
            emoji: Emoji,
        ) -> DispatchResult {
            let reactor = ensure_signed(origin)?;

            Self::ensure_valid_emoji(emoji)?;
            let item_owner = Self::get_item_and_validate_revision(&item_id, revision_id)?.owner;

            let mut added = false;
            ItemAccountReactions::<T>::try_mutate_exists(
                (&item_id, &revision_id, &reactor),
                |maybe_reactions| -> DispatchResult {
                    let reactions = maybe_reactions.get_or_insert_with(BoundedVec::default);
                    if reactions.contains(&emoji) {
                        return Ok(());
                    }
                    reactions
                        .try_push(emoji)
                        .map_err(|_| Error::<T>::TooManyEmojis)?;
                    added = true;
                    Ok(())
                },
            )?;

            if added {
                Self::deposit_event(Event::AddReaction {
                    item_id,
                    revision_id,
                    item_owner,
                    reactor,
                    emoji,
                });
            }

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_reaction())]
        pub fn remove_reaction(
            origin: OriginFor<T>,
            item_id: ItemId,
            revision_id: RevisionId,
            emoji: Emoji,
        ) -> DispatchResult {
            let reactor = ensure_signed(origin)?;

            Self::ensure_valid_emoji(emoji)?;
            let item_owner = Self::get_item_and_validate_revision(&item_id, revision_id)?.owner;

            let mut removed = false;
            ItemAccountReactions::<T>::mutate_exists(
                (&item_id, &revision_id, &reactor),
                |maybe_reactions| {
                    let Some(reactions) = maybe_reactions.as_mut() else {
                        return;
                    };

                    if let Some(index) = reactions.iter().position(|stored| stored == &emoji) {
                        reactions.remove(index);
                        removed = true;
                        if reactions.is_empty() {
                            *maybe_reactions = None;
                        }
                    }
                },
            );

            if removed {
                Self::deposit_event(Event::RemoveReaction {
                    item_id,
                    revision_id,
                    item_owner,
                    reactor,
                    emoji,
                });
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_valid_emoji(emoji: Emoji) -> Result<(), Error<T>> {
            ensure!(emoji.0 != 0, Error::<T>::InvalidEmoji);
            ensure!(char::from_u32(emoji.0).is_some(), Error::<T>::InvalidEmoji);
            Ok(())
        }

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
        AddReaction {
            item_id: ItemId,
            revision_id: RevisionId,
            item_owner: T::AccountId,
            reactor: T::AccountId,
            emoji: Emoji,
        },
        RemoveReaction {
            item_id: ItemId,
            revision_id: RevisionId,
            item_owner: T::AccountId,
            reactor: T::AccountId,
            emoji: Emoji,
        },
    }

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
        /// The account has reached the maximum number of emoji reactions for the item.
        TooManyEmojis,
    }

    #[pallet::storage]
    pub type ItemAccountReactions<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, ItemId>,
            NMapKey<Blake2_128Concat, RevisionId>,
            NMapKey<Blake2_128Concat, T::AccountId>,
        ),
        ReactionsOf<T>,
        OptionQuery,
    >;
}
