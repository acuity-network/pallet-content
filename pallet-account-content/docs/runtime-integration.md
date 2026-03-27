# Runtime Integration

`pallet-account-content` is designed as a sibling pallet to `pallet-content`.

## Requirements

- your runtime must include `pallet-content`
- the runtime type used for `pallet-account-content` must also implement
  `pallet_content::Config`
- choose an appropriate `MaxItemsPerAccount` bound for your chain

## Example

```rust
impl pallet_content::Config for Runtime {
    type WeightInfo = pallet_content::SubstrateWeight<Runtime>;
    type ItemIdNamespace = frame_support::traits::ConstU32<0>;
    type MaxParents = frame_support::traits::ConstU32<64>;
    type MaxLinks = frame_support::traits::ConstU32<256>;
    type MaxMentions = frame_support::traits::ConstU32<256>;
}

impl pallet_account_content::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_account_content::SubstrateWeight<Runtime>;
    type MaxItemsPerAccount = frame_support::traits::ConstU32<1024>;
}
```

Add both pallets to `construct_runtime!` so `pallet-account-content` can resolve
ownership through `pallet-content` storage.
