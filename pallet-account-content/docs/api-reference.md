# API Reference

## Extrinsics

### `add_item`

```rust
add_item(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult
```

Adds `item_id` to the caller's list.

Fails when:

- the item is already in the caller's list (`ItemAlreadyAdded`)
- the referenced content item does not exist (`ItemNotFound`)
- the caller does not own the content item (`WrongAccount`)
- the account list is already full (`AccountItemsFull`)

Emits:

- `AddItem { account, item_id }`

### `remove_item`

```rust
remove_item(origin: OriginFor<T>, item_id: ItemId) -> DispatchResult
```

Removes `item_id` from the caller's list.

Fails when:

- the item is not in the caller's list (`ItemNotAdded`)
- the referenced content item no longer exists (`ItemNotFound`)
- the caller no longer owns the content item (`WrongAccount`)

Emits:

- `RemoveItem { account, item_id }`

## Storage

### `AccountItemIds`

```rust
StorageMap<AccountId, BoundedVec<ItemId, MaxItemsPerAccount>, ValueQuery>
```

Ordered list of item ids for each account.

### `AccountItemIdIndex`

```rust
StorageDoubleMap<AccountId, ItemId, u32, ValueQuery>
```

Membership and index lookup table storing `index + 1`. A value of `0` means the
item is not present in the account list.

## Helper methods

- `get_item_exists(account, item_id) -> bool`
- `get_item_count(account) -> u32`
- `get_all_items(account) -> BoundedVec<ItemId, MaxItemsPerAccount>`
- `get_item_exists_by_account(account, item_id) -> bool`
- `get_item_count_by_account(account) -> u32`
- `get_all_items_by_account(account) -> BoundedVec<ItemId, MaxItemsPerAccount>`

## Errors

- `ItemAlreadyAdded`
- `ItemNotAdded`
- `ItemNotFound`
- `WrongAccount`
- `AccountItemsFull`
- `IndexOverflow`
