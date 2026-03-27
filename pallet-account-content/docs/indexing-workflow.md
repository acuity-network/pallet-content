# Indexing Workflow

`pallet-account-content` keeps the canonical account-to-item relationship in
storage and emits compact lifecycle events for indexers.

Recommended indexer behavior:

- subscribe to `pallet_account_content::AddItem`
- subscribe to `pallet_account_content::RemoveItem`
- maintain a materialized account-to-content table keyed by `(account, item_id)`
- join against `pallet-content` item metadata when richer content context is
  needed

Because the referenced `ItemId` already belongs to `pallet-content`, indexers can
correlate ownership and published content history across both pallets.
