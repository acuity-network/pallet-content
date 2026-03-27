use super::*;
use crate::Pallet;
use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_support::traits::Get;
use polkadot_sdk::{frame_benchmarking, frame_support, frame_system, sp_io};
use sp_io::hashing::blake2_256;

#[benchmarks]
mod benchmarks {
    use super::*;
    use pallet_content::{IpfsHash, ItemId, Nonce, Pallet as Content, RevisionId};

    const REVISIONABLE: u8 = 1 << 0;
    const INITIAL_REVISION_ID: RevisionId = 0;
    const GRINNING_FACE: Emoji = Emoji(0x1F600);

    fn nonce_from_seed(seed: u32) -> Nonce {
        let mut bytes = [0u8; 32];
        bytes[..core::mem::size_of::<u32>()].copy_from_slice(&seed.to_le_bytes());
        Nonce::decode(&mut &bytes[..]).expect("32-byte nonce decodes")
    }

    fn emoji_from_seed(seed: u32) -> Emoji {
        Emoji(0x1F600 + seed)
    }

    fn publish_content_item<T: Config>(caller: &T::AccountId, seed: u32) -> ItemId {
        let nonce = nonce_from_seed(seed);
        assert_ok!(Content::<T>::publish_item(
            frame_system::RawOrigin::Signed(caller.clone()).into(),
            nonce.clone(),
            Default::default(),
            REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default(),
        ));

        let mut item_id = ItemId::default();
        item_id.0.copy_from_slice(&blake2_256(
            &[
                caller.encode(),
                nonce.encode(),
                <T as pallet_content::Config>::ItemIdNamespace::get().encode(),
            ]
            .concat(),
        ));
        item_id
    }

    #[benchmark]
    pub fn add_reaction() {
        let caller: T::AccountId = whitelisted_caller();
        let item_id = publish_content_item::<T>(&caller, 0);
        let max_emojis = T::MaxEmojis::get();
        assert!(max_emojis > 0, "benchmark requires MaxEmojis > 0");

        for seed in 0..max_emojis.saturating_sub(1) {
            assert_ok!(Pallet::<T>::add_reaction(
                frame_system::RawOrigin::Signed(caller.clone()).into(),
                item_id.clone(),
                INITIAL_REVISION_ID,
                emoji_from_seed(seed),
            ));
        }

        let emoji = emoji_from_seed(max_emojis - 1);

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
            INITIAL_REVISION_ID,
            emoji,
        );

        assert_eq!(
            ItemAccountReactions::<T>::get((&item_id, &INITIAL_REVISION_ID, &caller))
                .expect("reaction entry must exist")
                .len(),
            usize::try_from(max_emojis).expect("u32 fits in usize")
        );
    }

    #[benchmark]
    pub fn remove_reaction() {
        let caller: T::AccountId = whitelisted_caller();
        let item_id = publish_content_item::<T>(&caller, 0);
        let max_emojis = T::MaxEmojis::get();
        assert!(max_emojis > 0, "benchmark requires MaxEmojis > 0");

        for seed in 0..max_emojis {
            assert_ok!(Pallet::<T>::add_reaction(
                frame_system::RawOrigin::Signed(caller.clone()).into(),
                item_id.clone(),
                INITIAL_REVISION_ID,
                emoji_from_seed(seed),
            ));
        }

        let emoji = GRINNING_FACE;

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
            INITIAL_REVISION_ID,
            emoji,
        );

        let reactions = ItemAccountReactions::<T>::get((&item_id, &INITIAL_REVISION_ID, &caller));
        if max_emojis == 1 {
            assert_eq!(reactions, None);
        } else {
            assert_eq!(
                reactions
                    .expect("reaction entry must remain until last emoji is removed")
                    .len(),
                usize::try_from(max_emojis - 1).expect("u32 fits in usize")
            );
        }
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
