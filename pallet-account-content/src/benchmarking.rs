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
    use pallet_content::{IpfsHash, ItemId, Nonce, Pallet as Content};

    const REVISIONABLE: u8 = 1 << 0;

    fn nonce_from_seed(seed: u32) -> Nonce {
        let mut bytes = [0u8; 32];
        bytes[..core::mem::size_of::<u32>()].copy_from_slice(&seed.to_le_bytes());
        Nonce::decode(&mut &bytes[..]).expect("32-byte nonce decodes")
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
    pub fn add_item() {
        let caller: T::AccountId = whitelisted_caller();
        let max_items = T::MaxItemsPerAccount::get();
        assert!(max_items > 0, "benchmark requires MaxItemsPerAccount > 0");

        let mut item_ids = frame_support::BoundedVec::<ItemId, T::MaxItemsPerAccount>::default();
        for seed in 0..max_items {
            item_ids
                .try_push(publish_content_item::<T>(&caller, seed))
                .expect("published items fit in benchmark bounds");
        }

        let item_id = item_ids
            .last()
            .cloned()
            .expect("benchmark requires at least one content item");

        for existing_item_id in item_ids.iter().take(item_ids.len().saturating_sub(1)) {
            assert_ok!(Pallet::<T>::add_item(
                frame_system::RawOrigin::Signed(caller.clone()).into(),
                existing_item_id.clone(),
            ));
        }

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        assert_eq!(AccountItemIdIndex::<T>::get(&caller, &item_id), max_items);
        assert_eq!(
            u32::try_from(AccountItemIds::<T>::get(&caller).len()).unwrap_or(u32::MAX),
            max_items,
        );
    }

    #[benchmark]
    pub fn remove_item() {
        let caller: T::AccountId = whitelisted_caller();
        let max_items = T::MaxItemsPerAccount::get();
        assert!(max_items > 0, "benchmark requires MaxItemsPerAccount > 0");

        let mut item_ids = frame_support::BoundedVec::<ItemId, T::MaxItemsPerAccount>::default();
        for seed in 0..max_items {
            let item_id = publish_content_item::<T>(&caller, seed);
            assert_ok!(Pallet::<T>::add_item(
                frame_system::RawOrigin::Signed(caller.clone()).into(),
                item_id.clone(),
            ));
            item_ids
                .try_push(item_id)
                .expect("published items fit in benchmark bounds");
        }

        let item_id = item_ids
            .first()
            .cloned()
            .expect("benchmark requires at least one tracked item");

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        assert_eq!(AccountItemIdIndex::<T>::get(&caller, &item_id), 0);
        assert_eq!(
            u32::try_from(AccountItemIds::<T>::get(&caller).len()).unwrap_or(u32::MAX),
            max_items - 1,
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
