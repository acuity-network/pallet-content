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
    use scale_info::prelude::vec::Vec;

    const REVISIONABLE: u8 = 1 << 0;
    const INITIAL_REVISION_ID: RevisionId = 0;

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
    pub fn set_reactions() {
        let caller: T::AccountId = whitelisted_caller();
        let item_id = publish_content_item::<T>(&caller, 0);
        let max_emojis = T::MaxEmojis::get();
        assert!(max_emojis > 0, "benchmark requires MaxEmojis > 0");

        let reactions: ReactionsOf<T> = (0..max_emojis)
            .map(emoji_from_seed)
            .collect::<Vec<_>>()
            .try_into()
            .expect("emojis fit within MaxEmojis bound");

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
            INITIAL_REVISION_ID,
            reactions,
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
