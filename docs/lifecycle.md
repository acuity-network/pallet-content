# Item Lifecycle State Diagram

This document summarizes lifecycle transitions for an item as managed by
`pallet-content`.

The actual payload fields (`parents`, `links`, `mentions`, `ipfs_hash`) are emitted in events and
indexed off-chain. This diagram focuses on on-chain control state.

## Core state bits

```
REVISIONABLE bit (0x01)  -> can call publish_revision
RETRACTABLE bit (0x02)   -> can call retract_item
RETRACTED bit (0x04)     -> item has been retracted
```

## Initial publish states

```
Input flags to publish_item       On-chain initial flags
-------------------------------- ------------------
0                                0
REVISIONABLE                     1
RETRACTABLE                      2
REVISIONABLE | RETRACTABLE        3
```

Any other bits are rejected as `InvalidFlags`.

## Call behavior by state

Let `F` be current flags.

```text
F = 0
  - publish_revision -> fails: ItemNotRevisionable
  - retract_item      -> fails: ItemNotRetractable
  - set_not_revisionable -> fails: ItemNotRevisionable
  - set_not_retractable -> fails: ItemNotRetractable

F = REVISIONABLE (1)
  - publish_revision -> increments revision_id, stays 1
  - retract_item      -> fails: ItemNotRetractable
  - set_not_revisionable -> clears REVISIONABLE -> 0
  - set_not_retractable -> fails: ItemNotRetractable

F = RETRACTABLE (2)
  - publish_revision -> fails: ItemNotRevisionable
  - retract_item      -> sets RETRACTED (4)
  - set_not_revisionable -> fails: ItemNotRevisionable
  - set_not_retractable -> clears RETRACTABLE -> 0

F = REVISIONABLE | RETRACTABLE (3)
  - publish_revision -> increments revision_id, stays 3
  - retract_item      -> sets RETRACTED (4)
  - set_not_revisionable -> clears REVISIONABLE -> 2
  - set_not_retractable -> clears RETRACTABLE -> 1

F = RETRACTED (4)
  - publish_revision -> fails: ItemRetracted
  - retract_item      -> fails: ItemRetracted
  - set_not_revisionable -> fails: ItemNotRevisionable
  - set_not_retractable -> fails: ItemNotRetractable
```

## Transition diagram

```text
[Create: 0|1|2|3]
        |
        | publish_revision succeeds only when bit 0 is set
        v
     revision_id + 1

From 3 (rev+ret) -------------------------> 4 (retracted)
   | (set_not_revisionable)                  |
   v                                         |
2 (retractable only)                          |
   | (set_not_retractable)                    |
   v                                          |
0 (locked) ----------------------------------+

From 3 (rev+ret) -------------------------> 1 (revision only)
   | (set_not_retractable)
   v
   1 (revision only)

From 1 (revision only)
   | (publish_revision)
   v
   1 (revision only), revision_id++
```

## Error outcomes to remember

- `ItemNotFound`: item does not exist
- `WrongAccount`: non-owner called any mutating extrinsic
- `InvalidFlags`: publish_item used unsupported bits
- `ItemAlreadyExists`: publish_item computed id already in storage
- `ItemRetracted`: publish_revision or retract_item used after retraction
- `RevisionIdOverflow`: revision_id increment wrapped
