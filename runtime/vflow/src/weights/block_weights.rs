// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 53.0.0
//! DATE: 2026-03-10 (Y/M/D)
//! HOSTNAME: `ef955f4b6c11`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//!
//! SHORT-NAME: `block`, LONG-NAME: `BlockExecution`, RUNTIME: `VFlow`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: `/data/benchmark/runtime/vflow/src/weights`
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`

// Executed Command:
//   /usr/local/bin/vflow-node
//   benchmark
//   overhead
//   --runtime
//   /app/vflow_runtime.compact.compressed.wasm
//   --genesis-builder=runtime
//   --weight-path=/data/benchmark/runtime/vflow/src/weights
//   --header=/data/benchmark/HEADER-APACHE2
//   --warmup=10
//   --repeat=100
//   --base-path=/tmp/tmp.ULYpBzd15e

use sp_core::parameter_types;
use sp_weights::{constants::WEIGHT_REF_TIME_PER_NANOS, Weight};

parameter_types! {
    /// Weight of executing an empty block.
    /// Calculated by multiplying the *Average* with `1.0` and adding `0`.
    ///
    /// Stats nanoseconds:
    ///   Min, Max: 659_017, 718_336
    ///   Average:  672_065
    ///   Median:   671_860
    ///   Std-Dev:  7638.29
    ///
    /// Percentiles nanoseconds:
    ///   99th: 698_419
    ///   95th: 684_163
    ///   75th: 674_395
    pub const BlockExecutionWeight: Weight =
        Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(672_065), 3_630);
}

#[cfg(test)]
mod test_weights {
    use sp_weights::constants;

    /// Checks that the weight exists and is sane.
    // NOTE: If this test fails but you are sure that the generated values are fine,
    // you can delete it.
    #[test]
    fn sane() {
        let w = super::BlockExecutionWeight::get();

        // At least 100 µs.
        assert!(
            w.ref_time() >= 100u64 * constants::WEIGHT_REF_TIME_PER_MICROS,
            "Weight should be at least 100 µs."
        );
        // At most 50 ms.
        assert!(
            w.ref_time() <= 50u64 * constants::WEIGHT_REF_TIME_PER_MILLIS,
            "Weight should be at most 50 ms."
        );
    }
}
