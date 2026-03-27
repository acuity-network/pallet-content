# Lifecycle

For each account, profile state follows a simple lifecycle:

- absent
- set
- overwritten by a later set

## Set flow

`set_profile` validates current ownership in `pallet-content`, then stores the
new profile item id for the caller and emits `ProfileSet`.

The pallet does not provide a removal extrinsic, matching the original Solidity
contract.
