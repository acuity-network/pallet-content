# Runtime Integration

`pallet-content-reactions` is designed as a sibling pallet to `pallet-content`.

## Requirements

- your runtime must include `pallet-content`
- the runtime type used for `pallet-content-reactions` must also implement
  `pallet_content::Config`
- choose an appropriate `MaxEmojis` bound for your chain

## Example

```rust
impl pallet_content::Config for Runtime {
    type WeightInfo = pallet_content::SubstrateWeight<Runtime>;
    type ItemIdNamespace = frame_support::traits::ConstU32<0>;
    type MaxParents = frame_support::traits::ConstU32<64>;
    type MaxLinks = frame_support::traits::ConstU32<256>;
    type MaxMentions = frame_support::traits::ConstU32<256>;
}

impl pallet_content_reactions::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_content_reactions::SubstrateWeight<Runtime>;
    type MaxEmojis = frame_support::traits::ConstU32<16>;
}
```

Add both pallets to `construct_runtime!` so reactions can validate referenced
item and revision state through `pallet-content` storage.
