# API Reference

## Extrinsics

### `set_profile`

```rust
set_profile(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult
```

Sets `item_id` as the caller's current profile.

Fails when:

- the referenced content item does not exist (`ItemNotFound`)
- the caller does not own the content item (`WrongAccount`)

Emits:

- `ProfileSet { account, item_id }`

## Storage

### `AccountProfile`

```rust
StorageMap<AccountId, ItemId, OptionQuery>
```

Current profile item id for each account.

## Errors

- `ItemNotFound`
- `WrongAccount`
