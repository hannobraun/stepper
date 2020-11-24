#!/usr/bin/env bash
set -e

cargo build --verbose &&
cargo test --verbose
