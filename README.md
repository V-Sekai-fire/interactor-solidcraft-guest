# interactor-solidcraft-guest

# SolidCraft engine as a godot-sandbox (libriscv) guest.

# A thin Rust glue crate: registers the SolidCraft control-channel API as
# host-callable guest functions, builds a box, and halts so the host can call
# the API with `vmcall`. The engine itself is the sibling `bherbruck/solidcraft`
# checkout (`3-interactor/solidcraft`).

# Source of truth for the guest's public host-callable surface: `sigs/
# solidcraft.sigs`.

# Build with the goal-manifest workspace (so the engine sibling and the
# RISC-V sysroot are checked out):

#     cargo +nightly build --release
#     cp target/riscv64gc-unknown-linux-gnu/release/solidcraft-guest guest.elf

# Run under the godot-sandbox host (see 3-interactor/mujoco-sandbox-demo):
# set the program, raise memory_max, then vmcall.

# Licence: Apache-2.0 (see LICENSE).
