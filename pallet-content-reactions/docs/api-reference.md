# API Reference

## Extrinsics

### `set_reactions`

```rust
set_reactions(origin: OriginFor<T>, item_id: ItemId, revision_id: RevisionId, reactions: ReactionsOf<T>) -> DispatchResult
```

Sets the caller's full reaction set for the requested content revision, replacing any prior reactions. The entire set is emitted in a single `SetReactions` event. No state is stored on-chain.

Fails when:

- the referenced content item does not exist (`ItemNotFound`)
- the content item has been retracted (`ItemRetracted`)
- the referenced revision does not exist (`RevisionNotFound`)
- any emoji value in `reactions` is not a valid non-zero Unicode scalar value (`InvalidEmoji`)
- the `reactions` set contains duplicate emojis (`DuplicateEmoji`)
- the `reactions` set exceeds `MaxEmojis` (enforced by the `BoundedVec` type at decode time)

Emits:

- `SetReactions { item_id, revision_id, item_owner, reactor, reactions }` unconditionally on success

## Types

- `Emoji(u32)`
- `ReactionsOf<T> = BoundedVec<Emoji, MaxEmojis>`

## Errors

- `ItemNotFound`
- `ItemRetracted`
- `RevisionNotFound`
- `InvalidEmoji`
- `DuplicateEmoji`