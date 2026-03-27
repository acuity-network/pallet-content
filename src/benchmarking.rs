use super::*;
use crate::Pallet;
use codec::Encode;
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use polkadot_sdk::{frame_benchmarking, frame_support, frame_system, sp_io};
use sp_io::hashing::blake2_256;

#[benchmarks]
mod benchmarks {
    use super::*;

    fn filled_bounded_vec<Item, Max>(item: Item) -> BoundedVec<Item, Max>
    where
        Item: Clone,
        Max: Get<u32>,
    {
        let mut values = BoundedVec::<Item, Max>::default();
        for _ in 0..Max::get() {
            assert!(values.try_push(item.clone()).is_ok());
        }
        values
    }

    fn make_item_id<T: Config>(account: &T::AccountId, nonce: &Nonce) -> ItemId {
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

    fn publish_base_item<T: Config>(caller: &T::AccountId, nonce: Nonce, flags: u8) -> ItemId {
        assert_ok!(Pallet::<T>::publish_item(
            frame_system::RawOrigin::Signed(caller.clone()).into(),
            nonce.clone(),
            Default::default(),
            flags,
            Default::default(),
            Default::default(),
            IpfsHash::default(),
        ));
        make_item_id::<T>(caller, &nonce)
    }

    #[benchmark]
    pub fn publish_item() {
        let caller: T::AccountId = whitelisted_caller();
        let nonce = Nonce::default();
        let parents = filled_bounded_vec::<ItemId, T::MaxParents>(ItemId::default());
        let links = filled_bounded_vec::<ItemId, T::MaxLinks>(ItemId::default());
        let mentions = filled_bounded_vec::<T::AccountId, T::MaxMentions>(caller.clone());

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            nonce.clone(),
            parents.clone(),
            REVISIONABLE | RETRACTABLE,
            links.clone(),
            mentions.clone(),
            IpfsHash::default(),
        );

        let item_id = make_item_id::<T>(&caller, &nonce);
        assert!(ItemState::<T>::contains_key(item_id));
    }

    #[benchmark]
    pub fn publish_revision() {
        let caller: T::AccountId = whitelisted_caller();
        let nonce = Nonce::default();
        let item_id = publish_base_item::<T>(&caller, nonce, REVISIONABLE | RETRACTABLE);
        let links = filled_bounded_vec::<ItemId, T::MaxLinks>(ItemId::default());
        let mentions = filled_bounded_vec::<T::AccountId, T::MaxMentions>(caller.clone());

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
            links.clone(),
            mentions.clone(),
            IpfsHash::default(),
        );

        let item = ItemState::<T>::get(item_id).expect("item must exist");
        assert_eq!(item.revision_id, 1);
    }

    #[benchmark]
    pub fn retract_item() {
        let caller: T::AccountId = whitelisted_caller();
        let nonce = Nonce::default();
        let item_id = publish_base_item::<T>(&caller, nonce, REVISIONABLE | RETRACTABLE);

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        let item = ItemState::<T>::get(item_id).expect("item must exist");
        assert_eq!(item.flags, RETRACTED);
    }

    #[benchmark]
    pub fn set_not_revisionable() {
        let caller: T::AccountId = whitelisted_caller();
        let nonce = Nonce::default();
        let item_id = publish_base_item::<T>(&caller, nonce, REVISIONABLE | RETRACTABLE);

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        let item = ItemState::<T>::get(item_id).expect("item must exist");
        assert_eq!(item.flags & REVISIONABLE, 0);
    }

    #[benchmark]
    pub fn set_not_retractable() {
        let caller: T::AccountId = whitelisted_caller();
        let nonce = Nonce::default();
        let item_id = publish_base_item::<T>(&caller, nonce, REVISIONABLE | RETRACTABLE);

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Signed(caller.clone()),
            item_id.clone(),
        );

        let item = ItemState::<T>::get(item_id).expect("item must exist");
        assert_eq!(item.flags & RETRACTABLE, 0);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
