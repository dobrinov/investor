#!/usr/bin/env bash
# Builds the Rust core to WebAssembly for the browser app (web/src/lib/core-wasm).
# Needs rustup's toolchain with the wasm32-unknown-unknown target.
set -euo pipefail
cd "$(dirname "$0")/.."

toolchain="$(dirname "$(dirname "$(rustup which rustc)")")"
# rustup's rust-lld on macOS looks for libLLVM next to itself; point it at the toolchain.
export DYLD_FALLBACK_LIBRARY_PATH="$toolchain/lib${DYLD_FALLBACK_LIBRARY_PATH:+:$DYLD_FALLBACK_LIBRARY_PATH}"
export PATH="$toolchain/bin:$PATH"

wasm-pack build crates/wasm --target web --release --no-pack \
  --out-dir ../../web/src/lib/core-wasm --out-name core
