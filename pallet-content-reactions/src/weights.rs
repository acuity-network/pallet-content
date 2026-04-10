#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::{constants::RocksDbWeight, Weight}};
use polkadot_sdk::{frame_support, frame_system};

pub trait WeightInfo {
	fn set_reactions() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn set_reactions() -> Weight {
		Weight::from_parts(23_000_000, 5_400)
			.saturating_add(T::DbWeight::get().reads(2))
	}
}

impl WeightInfo for () {
	fn set_reactions() -> Weight {
		Weight::from_parts(23_000_000, 5_400)
			.saturating_add(RocksDbWeight::get().reads(2))
	}
}