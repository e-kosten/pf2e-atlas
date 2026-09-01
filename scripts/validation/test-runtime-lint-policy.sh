#!/bin/sh

set -eu

fixture="$(mktemp -d "${TMPDIR:-/tmp}/atlas-runtime-lint-fixture.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT HUP INT TERM
mkdir -p "$fixture/src"

cat >"$fixture/Cargo.toml" <<'EOF_CARGO'
[package]
name = "atlas-runtime-lint-fixture"
version = "0.0.0"
edition = "2024"

[workspace]
EOF_CARGO

cat >"$fixture/src/lib.rs" <<'EOF_RUNTIME_FAILURE'
pub fn recoverable(value: Option<u8>) -> u8 {
    value.unwrap()
}
EOF_RUNTIME_FAILURE

if cargo clippy --manifest-path "$fixture/Cargo.toml" --lib -- \
  -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo \
  -D clippy::unreachable >/dev/null 2>&1
then
  echo "runtime lint fixture accepted panic-oriented production code" >&2
  exit 1
fi

cat >"$fixture/src/lib.rs" <<'EOF_TEST_ALLOWED'
pub fn recoverable(value: Option<u8>) -> u8 {
    value.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    fn maybe(value: u8) -> Option<u8> {
        (value > 0).then_some(value)
    }

    #[test]
    fn assertion_oriented_test_code_remains_allowed() {
        let value = maybe(7).unwrap();
        assert_eq!(value, 7);
        if value == 0 {
            panic!("fixture assertion");
        }
    }
}
EOF_TEST_ALLOWED

cargo clippy --manifest-path "$fixture/Cargo.toml" --lib -- \
  -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo \
  -D clippy::unreachable >/dev/null
cargo clippy --manifest-path "$fixture/Cargo.toml" --tests -- \
  -D warnings \
  -D clippy::dbg_macro >/dev/null

echo "Runtime lint policy fixture passed."
