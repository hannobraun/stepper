#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

# Main library and drivers
cargo build --verbose &&
cargo test --verbose &&
cargo doc &&

# generate-drivers task
cargo task generate-drivers
