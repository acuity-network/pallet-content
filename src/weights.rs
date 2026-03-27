#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use polkadot_sdk::{frame_support, frame_system};
use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for this pallet.
pub trait WeightInfo {
	fn publish_item(parents: u32, links: u32, mentions: u32) -> Weight;
	fn publish_revision(links: u32, mentions: u32) -> Weight;
	fn retract_item() -> Weight;
	fn set_not_revisionable() -> Weight;
	fn set_not_retractable() -> Weight;
}

/// Handwritten placeholder weights for this pallet.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn publish_item(parents: u32, links: u32, mentions: u32) -> Weight {
		Weight::from_parts(32_000_000, 3_700)
			.saturating_add(Weight::from_parts(120_000, 0).saturating_mul(parents.into()))
			.saturating_add(Weight::from_parts(95_000, 0).saturating_mul(links.into()))
			.saturating_add(Weight::from_parts(70_000, 0).saturating_mul(mentions.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}

	fn publish_revision(links: u32, mentions: u32) -> Weight {
		Weight::from_parts(18_500_000, 3_700)
			.saturating_add(Weight::from_parts(95_000, 0).saturating_mul(links.into()))
			.saturating_add(Weight::from_parts(70_000, 0).saturating_mul(mentions.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}

	fn retract_item() -> Weight {
		Weight::from_parts(12_500_000, 3_700)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}

	fn set_not_revisionable() -> Weight {
		Weight::from_parts(11_800_000, 3_700)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}

	fn set_not_retractable() -> Weight {
		Weight::from_parts(11_800_000, 3_700)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn publish_item(parents: u32, links: u32, mentions: u32) -> Weight {
		Weight::from_parts(32_000_000, 3_700)
			.saturating_add(Weight::from_parts(120_000, 0).saturating_mul(parents.into()))
			.saturating_add(Weight::from_parts(95_000, 0).saturating_mul(links.into()))
			.saturating_add(Weight::from_parts(70_000, 0).saturating_mul(mentions.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}

	fn publish_revision(links: u32, mentions: u32) -> Weight {
		Weight::from_parts(18_500_000, 3_700)
			.saturating_add(Weight::from_parts(95_000, 0).saturating_mul(links.into()))
			.saturating_add(Weight::from_parts(70_000, 0).saturating_mul(mentions.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}

	fn retract_item() -> Weight {
		Weight::from_parts(12_500_000, 3_700)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}

	fn set_not_revisionable() -> Weight {
		Weight::from_parts(11_800_000, 3_700)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}

	fn set_not_retractable() -> Weight {
		Weight::from_parts(11_800_000, 3_700)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
