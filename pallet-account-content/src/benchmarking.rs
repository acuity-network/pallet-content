use super::*;
use crate::Pallet;
use codec::Encode;
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

    fn publish_content_item<T: Config>(caller: &T::AccountId, nonce: Nonce) -> ItemId {
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
        let item_id = publish_content_item::<T>(&caller, Nonce::default());

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        assert_eq!(AccountItemIdIndex::<T>::get(&caller, &item_id), 1);
    }

    #[benchmark]
    pub fn remove_item() {
        let caller: T::AccountId = whitelisted_caller();
        let item_id = publish_content_item::<T>(&caller, Nonce::default());
        assert_ok!(Pallet::<T>::add_item(
            frame_system::RawOrigin::Signed(caller.clone()).into(),
            item_id.clone(),
        ));

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        assert_eq!(AccountItemIdIndex::<T>::get(&caller, &item_id), 0);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
