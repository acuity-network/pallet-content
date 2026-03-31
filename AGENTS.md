# AI Agent Instructions

## Dependency policy

- Always use the `polkadot-sdk` crate for FRAME/Substrate dependencies in this repository.

## Weight policy

- Treat the handwritten weights in this repository as placeholders only.
- Do not spend time tuning or auditing pallet weights here; these pallets will be benchmarked after runtime integration, and the integrated runtime benchmarks are the source of truth.
