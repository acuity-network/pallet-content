use super::*;
use crate::Pallet;
use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    pub fn publish_item() {
        // Setup code

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Root,
            Nonce::default(),
            vec![],
            0,
            vec![],
            IpfsHash::default(),
        );
    }

    #[benchmark]
    pub fn publish_revision() {
        // Setup code
        let item_id = ItemId([
            2, 171, 77, 116, 200, 110, 195, 179, 153, 122, 79, 173, 243, 62, 85, 232, 39, 150, 80,
            200, 83, 158, 166, 126, 5, 60, 2, 220, 44, 253, 243, 52,
        ]);

        #[extrinsic_call]
        _(
            frame_system::RawOrigin::Root,
            item_id,
            vec![],
            IpfsHash::default(),
        );
    }

    #[benchmark]
    pub fn retract_item() {
        // Setup code
        let item_id = ItemId([
            2, 171, 77, 116, 200, 110, 195, 179, 153, 122, 79, 173, 243, 62, 85, 232, 39, 150, 80,
            200, 83, 158, 166, 126, 5, 60, 2, 220, 44, 253, 243, 52,
        ]);

        #[extrinsic_call]
        _(frame_system::RawOrigin::Root, item_id);
    }

    #[benchmark]
    pub fn set_not_revisionable() {
        // Setup code
        let item_id = ItemId([
            2, 171, 77, 116, 200, 110, 195, 179, 153, 122, 79, 173, 243, 62, 85, 232, 39, 150, 80,
            200, 83, 158, 166, 126, 5, 60, 2, 220, 44, 253, 243, 52,
        ]);

        #[extrinsic_call]
        _(frame_system::RawOrigin::Root, item_id);
    }

    #[benchmark]
    pub fn set_not_retractable() {
        // Setup code
        let item_id = ItemId([
            2, 171, 77, 116, 200, 110, 195, 179, 153, 122, 79, 173, 243, 62, 85, 232, 39, 150, 80,
            200, 83, 158, 166, 126, 5, 60, 2, 220, 44, 253, 243, 52,
        ]);

        #[extrinsic_call]
        _(frame_system::RawOrigin::Root, item_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
