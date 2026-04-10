# Lifecycle

For each `(item_id, revision_id, reactor)` tuple, the reaction set is the set of
emojis most recently submitted via `set_reactions`.

## Set flow

`set_reactions` validates that every emoji value is a valid non-zero Unicode
scalar value, checks for duplicates within the set, and verifies that the
target item and revision still exist. On success it emits a single
`SetReactions` event containing the full reaction set.

Because no state is stored on-chain, each call completely replaces the prior
reaction set for that tuple. An indexer should treat the latest `SetReactions`
event for a given `(item_id, revision_id, reactor)` as the current state.