# Indexing Workflow

`pallet-content-reactions` emits stateless events that describe the full
reaction set for each `(item_id, revision_id, reactor)` tuple.

Recommended indexer behavior:

- subscribe to `pallet_content_reactions::SetReactions`
- for each event, upsert a reaction set keyed by `(item_id, revision_id, reactor)`
  with the full `reactions` BoundedVec as the current state
- join against `pallet-content` when item ownership or revision metadata is
  needed

Because `SetReactions` always contains the complete reaction set, indexers need
not track incremental adds or removes — the latest event for a given key is
the source of truth.

Because events include both `item_owner` and `reactor`, downstream systems can
build owner-centric and audience-centric reaction views without extra on-chain
reads.