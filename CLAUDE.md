# CLAUDE.md

## Project Overview

VFlow is an EVM parachain built on the zkVerify relay chain using the Substrate/Polkadot SDK. It bridges VFY tokens between zkVerify and EVM chains via LayerZero. Contract deployment is permissioned (technical committee only); other EVM interactions are open.

## Project Structure

- `node/` — Parachain node binary (CLI, service, RPC, chain specs)
- `pallets/` — Custom FRAME pallets (`deployment-permissions`, `parachain-inherent`)
- `runtime/vflow/` — Main runtime (configs, weights, precompiles, tests)
- `runtime/common/` — Shared runtime code
- `tests/` — TypeScript functional tests (vitest, ethers, polkadot.js)
- `scripts/` — Dev scripts (zombienet, benchmarking)
- `ci/` — CI helper scripts
- `docker/` — Dockerfiles and compose configs

## Build & Test Commands

Uses `cargo-make` for task automation (`Makefile.toml`):

```bash
cargo make build          # Release build
cargo make test           # Run all tests (release, all features)
cargo make format         # cargo fmt
cargo make clippy         # Lint (--deny warnings)
cargo make check          # cargo check --all-features (SKIP_WASM_BUILD=1)
cargo make audit          # Security audit
cargo make machete        # Unused dependency check
cargo make zepter-fix     # Fix Cargo.toml feature propagation
cargo make ci             # All checks: format, zepter, check, build, test, clippy, audit, machete
```

Direct cargo commands:

```bash
cargo test --release --all-features                           # All tests
cargo test --lib -p vflow-runtime --release                   # Runtime tests (mainnet)
cargo test --lib -p vflow-runtime --features volta --release  # Runtime tests (volta testnet)
cargo build --release --features runtime-benchmarks           # Build with benchmarks
```

## Code Conventions

- **Rust edition**: 2021
- **Formatting**: Standard `cargo fmt` (no custom rustfmt config)
- **Clippy**: correctness=deny, suspicious=deny, complexity=deny, style=warn. `large_enum_variant`, `too_many_arguments`, `type_complexity` are allowed.
- **Lints**: All crates inherit workspace lints via `[lints] workspace = true`
- **Feature management**: Run `zepter run fix` to keep Cargo.toml feature propagation consistent

## Copyright Header

All Rust source files must include:

```rust
// Copyright 2025, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
```

## Dual-Chain Configuration

- **Mainnet** — Default (no feature flags)
- **Volta** — Test chain (enabled with `--features volta`)

Per-chain configs live in `runtime/vflow/src/configs/` (`mainnet.rs`, `volta.rs`).

## Key Dependencies

- **Substrate/Polkadot SDK** (stable2412) — Blockchain framework
- **Cumulus** — Parachain consensus and relay chain integration
- **Frontier** (Moonbeam fork, `moonbeam-polkadot-stable2412`) — EVM compatibility layer
- **zkVerify** — Relay chain (`v1.1.0-20251212`)

## Testing Patterns

- Unit tests live alongside code in `#[cfg(test)]` modules
- Runtime tests are in `runtime/vflow/src/tests/`
- Pallet tests use mock runtimes (`mock.rs` + `tests.rs`)
- Benchmarks require the `runtime-benchmarks` feature flag
- Weight files are auto-generated in `runtime/vflow/src/weights/`
