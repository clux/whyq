[private]
default:
  @just --list --unsorted

fmt:
  cargo fmt

test:
  cargo test

test-integration:
  #!/usr/bin/env bash
  cargo install --path .
  alias yq=whyq
  export RUST_LOG=debug
  bats test

release:
  cargo release minor --execute
