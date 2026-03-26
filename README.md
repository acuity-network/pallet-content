# pallet-content

Content pallet for a Substrate runtime.

## Security auditing

This repository uses `cargo-audit` with a project-local policy file at
`.cargo/audit.toml`.

- Known transitive advisories from pinned upstream dependencies are tracked via
  explicit `ignore` entries.
- The policy is intentionally strict (`deny = ["warnings"]`) so newly introduced
  advisories fail audit checks unless explicitly reviewed.
- Yanked crate checks are currently disabled because the affected crates are
  transitive from the pinned SDK set.

### Run audit locally

```bash
cargo audit
```

### Policy maintenance

When updating `polkadot-sdk` dependencies, re-run `cargo audit` and remove any
ignore entries that are no longer needed.
