use crate::Pallet;
use codec::Encode;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use polkadot_sdk::{frame_benchmarking, frame_support, frame_system, sp_io};
use sp_io::hashing::blake2_256;

use super::*;

#[benchmarks]
mod benchmarks {
    use super::*;
    use frame_support::assert_ok;
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
    pub fn set_profile() {
        let caller: T::AccountId = whitelisted_caller();
        let item_id = publish_content_item::<T>(&caller, Nonce::default());

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        assert_eq!(AccountProfile::<T>::get(caller), Some(item_id));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
