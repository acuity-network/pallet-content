# pallet-content-reactions

`pallet-content-reactions` is a Substrate FRAME pallet for account-scoped emoji
reactions on `pallet-content` revisions.

It preserves a compact, indexer-friendly model:

- reactions are keyed by `(item_id, revision_id, reactor)`
- each account stores a bounded set of distinct emoji values per revision
- reaction state is validated against live `pallet-content` ownership and
  revision state
- duplicate adds and missing removals are safe no-ops

## Storage

- `ItemAccountReactions: StorageNMap<(ItemId, RevisionId, AccountId), ReactionsOf<T>, OptionQuery>`

## Extrinsics

- `add_reaction(origin, item_id, revision_id, emoji)`
  - requires a valid non-zero Unicode scalar value
  - requires the referenced content item to exist and not be retracted
  - requires the referenced revision to exist
  - appends the emoji if it is not already present for the caller
  - emits `AddReaction` only when state changes

- `remove_reaction(origin, item_id, revision_id, emoji)`
  - requires a valid non-zero Unicode scalar value
  - requires the referenced content item to exist and not be retracted
  - requires the referenced revision to exist
  - removes the emoji if it is present for the caller
  - emits `RemoveReaction` only when state changes

## Types

- `Emoji(u32)`
  - stores a Unicode scalar value as a raw `u32`
  - must decode to a valid non-zero `char`

- `ReactionsOf<T>`
  - bounded vector of emoji values with capacity `MaxEmojis`

## Errors

- `ItemNotFound`
- `ItemRetracted`
- `RevisionNotFound`
- `InvalidEmoji`
- `TooManyEmojis`

## Runtime integration

`pallet-content-reactions` depends on `pallet-content` and its `Config` trait, so
the runtime must configure both pallets:

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

Then include both pallets in `construct_runtime!`.

### Additional documentation

- [API Reference](docs/api-reference.md)
- [Indexing Workflow](docs/indexing-workflow.md)
- [Runtime Integration Guide](docs/runtime-integration.md)
- [Lifecycle](docs/lifecycle.md)
