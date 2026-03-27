# API Reference

## Extrinsics

### `add_reaction`

```rust
add_reaction(origin: OriginFor<T>, item_id: ItemId, revision_id: RevisionId, emoji: Emoji) -> DispatchResult
```

Adds `emoji` to the caller's reaction set for the requested content revision.

Fails when:

- the referenced content item does not exist (`ItemNotFound`)
- the content item has been retracted (`ItemRetracted`)
- the referenced revision does not exist (`RevisionNotFound`)
- the provided emoji is not a valid non-zero Unicode scalar value (`InvalidEmoji`)
- the caller has already reached `MaxEmojis` distinct reactions for the revision (`TooManyEmojis`)

Emits:

- `AddReaction { item_id, revision_id, item_owner, reactor, emoji }` when the
  reaction set changes

### `remove_reaction`

```rust
remove_reaction(origin: OriginFor<T>, item_id: ItemId, revision_id: RevisionId, emoji: Emoji) -> DispatchResult
```

Removes `emoji` from the caller's reaction set for the requested content
revision.

Fails when:

- the referenced content item does not exist (`ItemNotFound`)
- the content item has been retracted (`ItemRetracted`)
- the referenced revision does not exist (`RevisionNotFound`)
- the provided emoji is not a valid non-zero Unicode scalar value (`InvalidEmoji`)

Emits:

- `RemoveReaction { item_id, revision_id, item_owner, reactor, emoji }` when the
  reaction set changes

## Storage

### `ItemAccountReactions`

```rust
StorageNMap<(ItemId, RevisionId, AccountId), ReactionsOf<T>, OptionQuery>
```

Stores the distinct emoji reactions attached by one account to one item revision.

## Types

- `Emoji(u32)`
- `ReactionsOf<T> = BoundedVec<Emoji, MaxEmojis>`

## Errors

- `ItemNotFound`
- `ItemRetracted`
- `RevisionNotFound`
- `InvalidEmoji`
- `TooManyEmojis`
