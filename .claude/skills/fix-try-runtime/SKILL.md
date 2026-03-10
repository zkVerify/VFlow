---
name: Fix Try-Runtime
description: >
  This skill should be used when the user asks to "fix try-runtime",
  "fix cargo make try-runtime", "debug try-runtime", "fix runtime upgrade",
  "fix migration", "fix on_runtime_upgrade", or mentions try-runtime failures,
  migration errors, or runtime upgrade issues. Provides a systematic workflow
  for diagnosing and fixing try-runtime compilation errors and execution
  failures.
---

# Fix Try-Runtime

Systematic workflow for diagnosing and fixing `cargo make try-runtime`
failures. The try-runtime tool simulates a runtime upgrade against a live
chain's state to verify that migrations execute correctly and storage
remains consistent.

## Important

- Try-runtime tests the **runtime upgrade path** — it fetches live state
  from a running chain and applies the new runtime on top of it.
- Fixes should never change production migration logic without careful
  review. Prefer fixing compilation issues, feature propagation, or
  configuration over altering migration behavior.
- The Makefile currently only tests **Volta** (testnet). CI tests both
  Volta and mainnet via separate matrix entries.

## How Try-Runtime Works

The `cargo make try-runtime` task does three things:

1. **Installs the CLI** (`try-runtime-inst`): installs `try-runtime-cli`
   from the paritytech repo at a pinned version (currently `v0.10.1`).
2. **Builds the runtime** (`build-try-runtime`): compiles `vflow-runtime`
   with `--features try-runtime,volta`.
3. **Runs the upgrade simulation** (`try-runtime`): executes
   `on-runtime-upgrade` against the live Volta chain state via WSS RPC.

## Workflow Overview

1. Build with try-runtime enabled and capture errors
2. Classify each error as compilation, CLI, or runtime execution failure
3. Apply the appropriate fix strategy
4. Verify the fix

## Step 1: Build with Try-Runtime

```bash
cargo build -p vflow-runtime --release --features try-runtime,volta 2>&1
```

If this fails, you have **compilation errors** — go to Step 2a.

If the build succeeds, go to Step 1b.

## Step 1b: Create a State Snapshot

Before running the upgrade simulation, create a snapshot of the live
chain state. This avoids re-downloading the full state on every attempt,
which can take several minutes.

```bash
try-runtime create-snapshot --uri wss://vflow-volta-rpc.zkverify.io volta-snapshot.snap
```

For mainnet:

```bash
try-runtime create-snapshot --uri wss://vflow-rpc.zkverify.io mainnet-snapshot.snap
```

**CLI note:** The snapshot path is a positional argument after `--uri`.
If omitted, the CLI auto-generates a name like
`tvflow-runtime-<version>@latest.snap`.

The snapshot file can be reused for all subsequent runs during the
debugging session, making iteration much faster.

## Step 1c: Run the Upgrade Simulation

Use the snapshot instead of the live RPC endpoint:

```bash
try-runtime --runtime ./target/release/wbuild/vflow-runtime/vflow_runtime.compact.compressed.wasm \
  on-runtime-upgrade --blocktime 6 --disable-mbm-checks --disable-spec-version-check \
  snap -p volta-snapshot.snap 2>&1
```

If the execution fails, go to Step 2b or 2c depending on the error type.

## Step 2a: Fix Compilation Errors

### Common Causes

**Missing `try-runtime` feature propagation:**
When a new pallet is added to the runtime, its `try-runtime` feature must
be listed in `runtime/vflow/Cargo.toml` under the `[features]` section's
`try-runtime` list. Similarly, if the pallet is used in `runtime/common`,
add it to `runtime/common/Cargo.toml`'s `try-runtime` feature list.

Check both files:
- `runtime/vflow/Cargo.toml` — `try-runtime = [...]` (line ~232)
- `runtime/common/Cargo.toml` — `try-runtime = [...]` (line ~83)

The pattern is: every pallet listed in `construct_runtime!` that supports
the `try-runtime` feature must appear in the feature propagation list.

**API changes in `frame-try-runtime`:**
After a polkadot-sdk upgrade, the `TryRuntime` trait may have changed
signatures. Check the `impl frame_try_runtime::TryRuntime<Block>` block
in `runtime/vflow/src/lib.rs` (around line 694) and compare against the
upstream trait definition.

**Missing `frame-try-runtime` dependency:**
Ensure `frame-try-runtime` is listed as an optional dependency in
`runtime/vflow/Cargo.toml` and included in both the `std` and
`try-runtime` feature lists.

### Quick Diagnostic: Feature Propagation

To check if all pallets have try-runtime propagated, compare the pallets
in `construct_runtime!` against the `try-runtime` feature list:

1. Read `runtime/vflow/src/lib.rs` to find all pallets in
   `construct_runtime!`
2. Read the `try-runtime` feature list in `runtime/vflow/Cargo.toml`
3. Any pallet present in `construct_runtime!` but missing from the
   `try-runtime` feature list is a likely culprit

Use `cargo-make`'s zepter tool to help catch propagation issues:

```bash
cargo make zepter-fix
```

## Step 2b: Fix CLI / Connection Errors

**try-runtime CLI version mismatch:**
The Makefile pins `try-runtime-cli` at `v0.10.1`. If the runtime uses a
newer polkadot-sdk version, the CLI may be incompatible. Check for errors
like:
- `Metadata version mismatch`
- `Cannot decode runtime version`
- `Unsupported metadata version`

Fix by updating the version tag in `Makefile.toml` (`try-runtime-inst`
task) to match the polkadot-sdk version. Find compatible versions at:
https://github.com/paritytech/try-runtime-cli/releases

CI uses a separate version (`try-runtime-version` input, currently
`v0.8.0` in `.github/workflows/CI-try-runtime.yml`). After updating
the Makefile version, also update the CI workflow default if needed.

**RPC connection failures:**
- `wss://vflow-volta-rpc.zkverify.io` — Volta testnet
- `wss://vflow-rpc.zkverify.io` — Mainnet

If the RPC endpoint is down or rate-limiting, try again later or use
a previously created snapshot file.

**State download timeout:**
Try-runtime downloads the full chain state. For large chains this can
take several minutes. If it times out, try with `--at <block_hash>` to
pin to a specific block. Better yet, create a snapshot (Step 1b) to
avoid repeated downloads.

## Step 2c: Fix Storage Version Mismatches

The most common execution failure is **storage version mismatch**. The
error looks like:

```
ERROR runtime::frame-support Session: On chain storage version StorageVersion(0)
  doesn't match in-code storage version StorageVersion(1).
```

This means a pallet has been upgraded (e.g. via a polkadot-sdk bump) and
its in-code storage version is now higher than on-chain, but the required
migration has not been wired into the runtime.

### Fix: Add the Migration to `runtime/vflow/src/migrations.rs`

The runtime uses a `migrations.rs` file with an `Unreleased` type alias
that lists all migrations for the next runtime upgrade. This tuple is
passed to `Executive` via `runtime/vflow/src/types.rs`.

**Architecture:**
- `runtime/vflow/src/migrations.rs` — defines `type Unreleased = (...)`
- `runtime/vflow/src/types.rs` — `Executive` uses `Unreleased` as its
  migrations parameter via `ct::Executive<..., Migrations>`
- `runtime/common/src/types.rs` — `Executive` accepts an optional
  `Migrations` generic (defaults to `()`)

**Steps to add a missing migration:**

1. Identify which pallet has the version mismatch from the error log.
2. Find the upstream migration type in `~/.cargo/git/checkouts/polkadot-sdk-*/`
   by searching for `MigrateV<FROM>ToV<TO>` in the pallet's source:
   ```bash
   grep -r "pub type MigrateV0ToV1" ~/.cargo/git/checkouts/polkadot-sdk-*/*/substrate/frame/<pallet>/src/
   ```
   For cumulus pallets, search under `cumulus/pallets/<pallet>/src/`.
3. Add the migration type to the `Unreleased` tuple in `migrations.rs`.
4. Some migrations require generic parameters (e.g. `pallet_session`'s
   `MigrateV0ToV1<T, S>` needs a `MigrateDisabledValidators` impl —
   use `InitOffenceSeverity<T>` as the default).

**Example `migrations.rs`:**
```rust
use crate::Runtime;

pub type Unreleased = (
    pallet_session::migrations::v1::MigrateV0ToV1<
        Runtime,
        pallet_session::migrations::v1::InitOffenceSeverity<Runtime>,
    >,
    cumulus_pallet_aura_ext::migration::MigrateV0ToV1<Runtime>,
);
```

After the migration has been released and executed on-chain, remove it
from the `Unreleased` tuple (try-runtime will warn: "migration can be
removed; on-chain is already at StorageVersion(N)").

## Step 2d: Fix Other Runtime Execution Failures

These occur when the runtime upgrade simulation runs but fails during
migration execution or post-upgrade checks.

### Common Execution Failure Patterns

**Migration panic / assertion failure:**
The error will show which migration or pallet's `on_runtime_upgrade`
failed. Read the panic message and backtrace to identify:
- Which pallet's migration panicked
- The specific assertion that failed
- Whether it's a pre-upgrade check, the migration itself, or a
  post-upgrade check

Locate the migration code:
- Custom migrations: `runtime/vflow/src/` or `pallets/<name>/src/`
- Upstream pallet migrations: check the pallet's `mod migration` or
  `fn on_runtime_upgrade` in the polkadot-sdk source

**`pre_upgrade` / `post_upgrade` hook failures:**
With the `try-runtime` feature enabled, pallets can implement
`pre_upgrade()` and `post_upgrade()` hooks that validate state before
and after migration. Failures here indicate:
- Pre-upgrade: the live chain state doesn't match what the migration
  expects (e.g., a storage item is missing or has an unexpected format)
- Post-upgrade: the migration didn't produce the expected result state

**Storage decode errors:**
If the live chain has storage in an old format that the new runtime
can't decode, you'll see decode errors. This usually means a storage
migration is needed but missing.

**Weight overflow:**
If the total migration weight exceeds the block weight limit, the
upgrade will fail. Check:
- `runtime/vflow/src/lib.rs` line ~697: the `on_runtime_upgrade`
  implementation returns `(weight, max_block_weight)`
- The `--blocktime 6` flag sets the expected block time
- Ensure migrations report accurate weights

### Debugging Migrations

To get more detail on what's happening during the upgrade, use the
snapshot with debug logging:

```bash
RUST_LOG=runtime=debug try-runtime \
  --runtime ./target/release/wbuild/vflow-runtime/vflow_runtime.compact.compressed.wasm \
  on-runtime-upgrade --blocktime 6 --disable-mbm-checks --disable-spec-version-check \
  snap -p volta-snapshot.snap 2>&1
```

The `runtime=debug` log level will show each pallet's migration hooks
executing, which helps pinpoint where the failure occurs. Using the
snapshot makes each iteration fast since there is no state download.

## Step 3: Verify the Fix

After applying fixes, rebuild and re-run against the snapshot:

```bash
cargo build -p vflow-runtime --release --features try-runtime,volta && \
try-runtime --runtime ./target/release/wbuild/vflow-runtime/vflow_runtime.compact.compressed.wasm \
  on-runtime-upgrade --blocktime 6 --disable-mbm-checks --disable-spec-version-check \
  snap -p volta-snapshot.snap
```

Once the fix is confirmed with the snapshot, do a final verification
against live state to ensure nothing is stale:

```bash
cargo make try-runtime
```

To test mainnet as well (matching CI):

```bash
cargo build -p vflow-runtime --release --features try-runtime && \
try-runtime --runtime ./target/release/wbuild/vflow-runtime/vflow_runtime.compact.compressed.wasm \
  on-runtime-upgrade --blocktime 6 --disable-mbm-checks --disable-spec-version-check \
  snap -p mainnet-snapshot.snap
```

## Key File Reference

| Purpose | Path |
|---------|------|
| Try-runtime task definitions | `Makefile.toml` (`build-try-runtime`, `try-runtime-inst`, `try-runtime`) |
| CI workflow (both chains) | `.github/workflows/CI-try-runtime.yml` |
| TryRuntime API impl | `runtime/vflow/src/lib.rs` (`impl TryRuntime<Block>`, ~line 694) |
| Runtime feature propagation | `runtime/vflow/Cargo.toml` (`try-runtime` feature list) |
| Common runtime feature propagation | `runtime/common/Cargo.toml` (`try-runtime` feature list) |
| construct_runtime! | `runtime/vflow/src/lib.rs` |
| Migrations (Unreleased) | `runtime/vflow/src/migrations.rs` (`type Unreleased`) |
| Executive type (uses Migrations) | `runtime/vflow/src/types.rs` |
| Custom migrations | `runtime/vflow/src/` (look for `mod migration` or migration structs) |
| Pallet migrations | `pallets/<name>/src/` |
| Genesis config presets | `runtime/vflow/src/genesis_config_presets.rs` |
| Per-chain configs | `runtime/vflow/src/configs/mainnet.rs`, `volta.rs` |
