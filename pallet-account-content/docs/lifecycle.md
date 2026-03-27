# Lifecycle

For each `(account, item_id)` pair, membership follows a simple lifecycle:

- absent
- added
- removed back to absent

## Add flow

`add_item` checks membership first, then validates current ownership in
`pallet-content`, then appends the item id and records `index + 1`.

## Remove flow

`remove_item` checks membership first, then validates ownership again, then swaps
the last item into the removed slot when needed before shrinking the list.

This preserves Solidity-compatible O(1) removal semantics.
