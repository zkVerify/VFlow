// Copyright 2024, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.

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

use std::future::Future;
use std::{
    process::{self, Command},
    time::Duration,
};

/// Run the given `future` and panic if the `timeout` is hit.
pub async fn run_with_timeout(timeout: Duration, future: impl Future<Output = ()>) {
    tokio::time::timeout(timeout, future)
        .await
        .expect("Hit timeout");
}

#[tokio::test]
async fn check_version() {
    run_with_timeout(Duration::from_secs(5), async move {
        let out = Command::new(assert_cmd::cargo::cargo_bin!("vflow-node"))
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .args([
                "--version",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        let v = std::env!("CARGO_PKG_VERSION");
        assert!(stdout.starts_with(&format!("vflow-node {v}")),
                "Version mismatch. Crate version = {v}, but node version string `{stdout}`. Curious: It SHOULD never happen because we're using the standard substrate cli here.");
    })
        .await
}
