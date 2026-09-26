#!/usr/bin/env bash
# Line coverage gate for heimdall-core (docs/port-plan.md §8: "heimdall-core >= 80% lines before
# feature UI work"). Requires cargo-llvm-cov and the llvm-tools rustup component:
#   rustup component add llvm-tools-preview
#   cargo install cargo-llvm-cov --locked
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo llvm-cov -p heimdall-core --fail-under-lines 80
