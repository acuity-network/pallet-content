# Lifecycle

For each `(item_id, revision_id, reactor, emoji)` tuple, membership follows a
simple lifecycle:

- absent
- added
- removed back to absent

## Add flow

`add_reaction` validates the emoji value, verifies that the target item and
revision still exist, then appends the emoji only if it is not already present.

## Remove flow

`remove_reaction` performs the same validation, then removes the emoji only if it
exists. When the last emoji is removed for a tuple, the storage entry is deleted.

This keeps storage compact while preserving idempotent add and remove behavior.
