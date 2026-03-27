# Indexing Workflow

`pallet-account-profile` keeps the canonical account-to-profile relationship in
storage and emits compact lifecycle events for indexers.

Recommended indexer behavior:

- subscribe to `pallet_account_profile::ProfileSet`
- maintain one current profile row keyed by `account`
- treat later `ProfileSet` events as overwrite updates
- join against `pallet-content` item metadata when richer profile context is
  needed

Because the referenced `ItemId` already belongs to `pallet-content`, indexers
can correlate account profiles with published content history across both
pallets.
