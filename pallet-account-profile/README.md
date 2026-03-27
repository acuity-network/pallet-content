# pallet-account-profile

`pallet-account-profile` is a Substrate FRAME pallet that lets an account keep a
single on-chain profile pointer to a `pallet-content` `ItemId` that it currently
owns.

It ports the Solidity contract `AcuityAccountProfile` into FRAME while
preserving its core behavior:

- one profile item per account
- profile updates overwrite the previous value
- ownership validation against `pallet-content`
- compact profile update events for indexers

## Storage

- `AccountProfile: StorageMap<AccountId, ItemId, OptionQuery>`

## Extrinsics

- `set_profile(origin, item_id)`
  - requires the item to exist in `pallet-content`
  - requires the signer to own the item
  - overwrites any existing profile for the signer
  - emits `ProfileSet`

## Runtime integration

`pallet-account-profile` depends on `pallet-content` and its `Config` trait, so
a runtime must configure both pallets:

```rust
impl pallet_content::Config for Runtime {
    type WeightInfo = pallet_content::SubstrateWeight<Runtime>;
    type MaxParents = frame_support::traits::ConstU32<64>;
    type MaxLinks = frame_support::traits::ConstU32<256>;
    type MaxMentions = frame_support::traits::ConstU32<256>;
}

impl pallet_account_profile::Config for Runtime {
    type WeightInfo = pallet_account_profile::SubstrateWeight<Runtime>;
}
```

Then include both pallets in `construct_runtime!`.
