# pallet-account-content

`pallet-account-content` is a Substrate FRAME pallet that lets an account keep an
on-chain list of `pallet-content` `ItemId`s that it currently owns.

It ports the Solidity contract `AcuityAccountItems` into FRAME while preserving
its core behavior:

- per-account ordered item lists
- O(1) membership checks through an index map
- O(1) removal by swap-with-last
- ownership validation against `pallet-content`

## Storage

- `AccountItemIds: StorageMap<AccountId, BoundedVec<ItemId, MaxItemsPerAccount>>`
- `AccountItemIdIndex: StorageDoubleMap<AccountId, ItemId, u32>` storing `index + 1`

## Extrinsics

- `add_item(origin, item_id)`
  - requires the item to exist in `pallet-content`
  - requires the signer to own the item
  - requires the item not already be in the signer's list
  - emits `AddItem`

- `remove_item(origin, item_id)`
  - requires the item already be in the signer's list
  - requires the signer to still own the item
  - removes the item with swap-with-last semantics
  - emits `RemoveItem`

## Query helpers

The pallet exposes helper methods mirroring the Solidity read API:

- `get_item_exists`
- `get_item_count`
- `get_all_items`
- `get_item_exists_by_account`
- `get_item_count_by_account`
- `get_all_items_by_account`

## Runtime integration

`pallet-account-content` depends on `pallet-content` and its `Config` trait, so a
runtime must configure both pallets:

```rust
impl pallet_content::Config for Runtime {
    type WeightInfo = pallet_content::SubstrateWeight<Runtime>;
    type MaxParents = frame_support::traits::ConstU32<64>;
    type MaxLinks = frame_support::traits::ConstU32<256>;
}

impl pallet_account_content::Config for Runtime {
    type WeightInfo = pallet_account_content::SubstrateWeight<Runtime>;
    type MaxItemsPerAccount = frame_support::traits::ConstU32<1024>;
}
```

Then include both pallets in `construct_runtime!`.
