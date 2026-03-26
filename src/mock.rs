//! Test environment for template pallet.

use crate::{self as pallet_content, Config};
use frame_support::derive_impl;
use polkadot_sdk::{frame_support, frame_system, pallet_balances, sp_io};
use sp_io::TestExternalities;

pub type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Content: pallet_content,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type Block = Block;
    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

impl Config for Test {
    type WeightInfo = ();
    type MaxParents = frame_support::traits::ConstU32<64>;
    type MaxLinks = frame_support::traits::ConstU32<256>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = TestExternalities::new(Default::default());
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}
