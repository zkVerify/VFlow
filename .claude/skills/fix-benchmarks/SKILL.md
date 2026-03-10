---
name: Fix Benchmarks
description: >
  This skill should be used when the user asks to "fix benchmarks",
  "fix cargo make test-bench", "fix benchmark compilation errors",
  "fix benchmark panics", "fix bench", or mentions broken benchmarks,
  benchmark failures, or pallet benchmark issues. Provides a systematic
  workflow for diagnosing and fixing benchmark compilation errors and
  runtime panics.
---

# Fix Benchmarks

Systematic workflow for diagnosing and fixing `cargo make test-bench`
failures. Covers two failure classes: compilation errors and runtime
panics.

## Important

- This skill fixes **existing** broken benchmarks only. It does NOT
  write benchmarks for new pallets or regenerate weight files.
- Benchmark fixes should never change production behavior. Use
  `#[cfg(feature = "runtime-benchmarks")]` guards when adjusting
  constants or configuration for benchmark compatibility.

## Workflow Overview

1. Build with benchmarks enabled and capture errors
2. Classify each error as a compilation failure or runtime panic
3. Apply the appropriate fix strategy
4. Verify the fix by re-running `cargo make test-bench`

## Step 1: Build with Benchmarks

```bash
cargo build --release --features runtime-benchmarks 2>&1
```

If this fails, you have **compilation errors** — go to Step 2a.

If the build succeeds, run the benchmarks:

```bash
./target/release/vflow-node benchmark pallet --pallet "*" --extrinsic "*" -s 2 -r 1 2>&1
```

If a benchmark panics, go to Step 2b.

## Step 2a: Fix Compilation Errors

Compilation errors typically come from pallet API changes in upstream
dependencies (polkadot-sdk, Frontier, cumulus) that break benchmark
code.

### Diagnosis

1. Read the compiler error to identify the affected crate and file
2. Common locations:
   - Pallet benchmarks: `pallets/<name>/src/benchmarking.rs`
   - Runtime benchmark list: `runtime/vflow/src/benchmark.rs`
   - Runtime benchmark dispatch: `runtime/vflow/src/lib.rs` (the
     `impl_runtime_apis!` block, `BenchmarkingApi` section)
   - XCM benchmark config: also in `runtime/vflow/src/lib.rs`

### Common Compilation Fix Patterns

**Missing or changed trait implementations:**
The upstream pallet added or changed a `Config` associated type that
the `BenchmarkHelper` or similar benchmark trait requires. Check what
the upstream benchmark expects and implement it in the runtime config.

**Changed benchmark function signatures:**
The upstream `#[benchmarks]` module changed its function signatures or
setup requirements. Read the new upstream benchmark code to understand
what changed.

**New pallet added to runtime but missing from benchmark list:**
Add the pallet to `runtime/vflow/src/benchmark.rs` inside the
`define_benchmarks!` macro, and ensure the `runtime-benchmarks` feature
is propagated in `runtime/vflow/Cargo.toml`.

### Where to Look for Upstream Reference

When fixing compilation errors, compare against the upstream benchmark
code from the dependency that changed. The key upstream sources are:

- **Polkadot SDK pallets** (stable2512): `frame/` and `pallets/`
  directories in polkadot-sdk
- **Frontier pallets** (Moonbeam fork, `moonbeam-polkadot-stable2512`):
  Moonbeam's frontier fork
- **Cumulus pallets**: `cumulus/pallets/` in polkadot-sdk

## Step 2b: Fix Runtime Panics

Runtime panics occur when benchmarks execute but hit assertion failures
or impossible states due to configuration mismatches between what the
upstream benchmark code expects and our runtime configuration.

### Diagnosis

1. Read the panic message and backtrace to identify:
   - Which pallet and extrinsic panicked
   - The assertion or condition that failed
2. Locate the upstream benchmark code for that pallet to understand
   what preconditions it expects

### Common Runtime Panic Fix Patterns

**Insufficient balances or existential deposit issues:**
Benchmarks may expect accounts to have minimum balances. Check:
- `runtime/common/src/constants.rs` — `EXISTENTIAL_DEPOSIT` is
  conditionally set under `runtime-benchmarks`. The value must balance
  two competing constraints:
  - **Non-zero**: Upstream benchmarks like `pallet_collator_selection`
    derive `CandidacyBond` from `Currency::minimum_balance()` (= ED)
    and assert it's > 0.
  - **Low enough**: With `insecure_zero_ed` always enabled on
    pallet-balances, the upstream benchmark code takes the zero-ED
    code path (`cfg!(feature = "insecure_zero_ed")`), which assumes
    accounts are NOT reaped. If ED is too high (e.g. 100), accounts
    with balance below ED get reaped, breaking assertions. ED must be
    smaller than the benchmark's hardcoded minimum balance (100) to
    avoid this. Currently set to `1`.
- `runtime/vflow/src/genesis_config_presets.rs` — The dev genesis
  endows accounts and adds extra accounts under `runtime-benchmarks`.
  Add more endowed accounts or increase `ENDOWMENT` if benchmarks need
  more funded accounts.

**Transaction fee / minimum balance mismatches:**
- `runtime/vflow/src/configs/monetary.rs` contains
  `OnChargeTransactionRuntimeBenchmarks`, a benchmark-only wrapper
  around the transaction payment handler. Adjust `minimum_balance()`
  if benchmarks fail due to fee-related assertions.

**Missing or invalid genesis state:**
Some benchmarks expect specific storage entries to exist at genesis.
Fix by adjusting `development_config_genesis()` in
`runtime/vflow/src/genesis_config_presets.rs`, guarded behind
`#[cfg(feature = "runtime-benchmarks")]`.

**Configuration constant too low or too high:**
Some benchmarks parametrize over runtime constants (e.g., max items,
max weight). If a constant is set too low for the benchmark to run,
add a benchmark-specific override using conditional compilation:

```rust
#[cfg(not(feature = "runtime-benchmarks"))]
pub const SOME_LIMIT: u32 = 10;

#[cfg(feature = "runtime-benchmarks")]
pub const SOME_LIMIT: u32 = 100;
```

Follow the established pattern in `runtime/common/src/constants.rs`.

**XCM benchmark configuration:**
XCM benchmarks have dedicated configuration in `runtime/vflow/src/lib.rs`
inside the `impl_runtime_apis!` block. Key types:
- `ExistentialDepositAsset` — minimum asset for XCM transfers. Must use
  a meaningful non-zero amount (e.g. `CENTS`), not `ExistentialDeposit::get()`
  which is near-zero under benchmarks.
- `XcmConfig` / `DeliveryHelper` — XCM routing for benchmarks

**AccountId32 vs AccountKey20 incompatibility (EVM chains):**
VFlow uses 20-byte EVM accounts (`AccountKey20`), but some upstream
pallet_xcm extrinsics (e.g. `add_authorized_alias`,
`remove_authorized_alias`) hardcode `AccountId32` pattern matching.
These benchmarks will always fail on EVM chains. The fix is to use
`frame_support::traits::Disabled` for `AuthorizedAliasConsideration`
in `pallet_xcm::Config` (in `runtime/vflow/src/configs/xcm.rs`). This
makes the benchmark fail gracefully at ticket creation with
`BenchmarkError::Override(Weight::MAX)` — the same approach Moonbeam
uses. The resulting weight file records `Weight::MAX`, effectively
disabling those extrinsics at runtime.

**Checking how Moonbeam handles a benchmark issue:**
When an upstream benchmark is incompatible with EVM/20-byte-account
chains, check Moonbeam's runtime for reference
(https://github.com/moonbeam-foundation/moonbeam). They face the same
`AccountKey20` constraints. Common patterns: using `Disabled` for
unsupported features, letting benchmarks fail gracefully via
`BenchmarkError::Override(Weight::MAX)`, or configuring benchmark
traits to return `None` to skip.

## Step 3: Verify the Fix

After applying fixes, run the full benchmark test:

```bash
cargo make test-bench
```

This builds with `--features runtime-benchmarks` and runs all pallet
benchmarks with minimal steps/repeats (`-s 2 -r 1`).

If only specific pallets were affected, you can verify faster by
targeting them:

```bash
cargo build --release --features runtime-benchmarks && \
./target/release/vflow-node benchmark pallet \
  --pallet "pallet_name" --extrinsic "*" -s 2 -r 1
```

## Key File Reference

| Purpose | Path |
|---------|------|
| Benchmark task definition | `Makefile.toml` (`test-bench`, `build-bench`) |
| Benchmark list (which pallets) | `runtime/vflow/src/benchmark.rs` |
| Benchmark dispatch & XCM config | `runtime/vflow/src/lib.rs` (`BenchmarkingApi`) |
| Conditional constants | `runtime/common/src/constants.rs` |
| Monetary / fee config | `runtime/vflow/src/configs/monetary.rs` |
| XCM pallet config | `runtime/vflow/src/configs/xcm.rs` (`AuthorizedAliasConsideration`, `ExecuteXcmOrigin`) |
| Dev genesis presets | `runtime/vflow/src/genesis_config_presets.rs` |
| Per-chain configs | `runtime/vflow/src/configs/mainnet.rs`, `volta.rs` |
| Pallet benchmarks | `pallets/<name>/src/benchmarking.rs` |
| Runtime feature propagation | `runtime/vflow/Cargo.toml` (`runtime-benchmarks` feature) |
