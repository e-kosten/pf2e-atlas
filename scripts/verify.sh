#!/bin/sh

set -eu

REPO_ROOT="$(git rev-parse --show-toplevel)"

cd "$REPO_ROOT"
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro
cargo clippy --workspace --lib --bins -- -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo \
  -D clippy::unreachable
cargo test --workspace
cargo build --workspace
