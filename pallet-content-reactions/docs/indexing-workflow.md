# Indexing Workflow

`pallet-content-reactions` keeps reaction state in storage and emits compact
change events for indexers.

Recommended indexer behavior:

- subscribe to `pallet_content_reactions::AddReaction`
- subscribe to `pallet_content_reactions::RemoveReaction`
- materialize a reaction table keyed by `(item_id, revision_id, reactor, emoji)`
- join against `pallet-content` when item ownership or revision metadata is
  needed

Because events include both `item_owner` and `reactor`, downstream systems can
build owner-centric and audience-centric reaction views without extra on-chain
reads.
