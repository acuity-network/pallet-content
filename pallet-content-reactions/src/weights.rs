#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::{constants::RocksDbWeight, Weight}};
use polkadot_sdk::{frame_support, frame_system};

pub trait WeightInfo {
	fn add_reaction() -> Weight;
	fn remove_reaction() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn add_reaction() -> Weight {
		Weight::from_parts(23_000_000, 5_400)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}

	fn remove_reaction() -> Weight {
		Weight::from_parts(22_000_000, 5_400)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

impl WeightInfo for () {
	fn add_reaction() -> Weight {
		Weight::from_parts(23_000_000, 5_400)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}

	fn remove_reaction() -> Weight {
		Weight::from_parts(22_000_000, 5_400)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
