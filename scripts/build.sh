#!/bin/sh
# Release build of the guest ELF with no build-machine paths in it.
#
# rustc writes source paths into the ELF (panic messages, the engine's
# error strings). Map the checkout's parent, the cargo registry and the
# toolchain to fixed names so the ELF says the same thing on every machine.
# Cargo's own trim-paths profile setting is still unstable (cargo 1.98).
#
# Usage: scripts/build.sh [cargo build args], e.g. --target-dir DIR.
# The flags go through --config so they join the target rustflags in
# .cargo/config.toml instead of replacing them, as RUSTFLAGS would.
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd)
parent=$(dirname "$repo")
cargo_home=${CARGO_HOME:-$HOME/.cargo}
rustup_home=${RUSTUP_HOME:-$HOME/.rustup}
cd "$repo"
exec cargo build --release \
	--config "target.riscv64gc-unknown-linux-gnu.rustflags=[\"--remap-path-prefix=$parent=/src\",\"--remap-path-prefix=$cargo_home/registry/src=/cargo\",\"--remap-path-prefix=$rustup_home=/rustup\"]" \
	"$@"
