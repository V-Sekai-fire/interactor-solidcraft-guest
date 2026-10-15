# interactor-solidcraft-guest

The SolidCraft engine exposed as a godot-sandbox (libriscv) guest: a thin
Rust glue crate that registers the SolidCraft control-channel API as
host-callable guest functions, then halts so the host can drive it with
`vmcall`. The engine itself is the sibling checkout `bherbruck/solidcraft`
(`3-interactor/solidcraft`), found as `../solidcraft` by the goal manifest.

## What it is for

`solidcraft-guest.elf` is a static RISC-V ELF guest that boots under the
`godot-sandbox` `Sandbox` addon and drives the pure-Rust `solidcraft-engine`
crate. This repo holds only the glue, the CI, and the manifest entries needed
to build the guest — the engine is a separate repository.

The guest registers three host-callable functions, declared in the form-A
`sigs/solidcraft.sigs` (source of truth for the public surface):

| function                     | args                                             | returns | note                             |
|------------------------------|--------------------------------------------------|---------|----------------------------------|
| `SolidCraft.solid_default`   | `( )`                                            | Array   | 10x20x30 mm box, canonical example |
| `SolidCraft.solid_box`       | `(float length, float width, float height)`      | Array   | parameterised box, mm            |
| `SolidCraft.solid_command_count` | `( )`                                        | int     | engine command-spec count        |

`solid_default` and `solid_box` return a compact inspection array:

```
[ok, body_count, vertex_count, triangle_count, minx, miny, minz, maxx, maxy, maxz]
```

## Build and run

Build inside the goal-manifest workspace so the engine sibling and the RISC-V
sysroot are checked out (this is how the manifest's build gate runs it):

```sh
cargo +nightly build --release
cp target/riscv64gc-unknown-linux-gnu/release/solidcraft-guest guest.elf
```

Run under the godot-sandbox host (see `3-interactor/mujoco-sandbox-demo`) by
setting the program, raising `memory_max`, then calling `vmcall`:

```gdscript
sb.set("program", load("res://plans/solidcraft-guest.elf"))
sb.set_memory_max(2048)
var r = sb.vmcall("SolidCraft.solid_default")
print(r)
```

## Layout

- `Cargo.toml` — glue crate (`solidcraft-guest`), depends on `solidcraft-engine`
  (`../solidcraft/crates/engine`) and `serde_json`.
- `src/main.rs` — registers the guest API and calls `fast_exit`.
- `sigs/solidcraft.sigs` — form-A `.sigs` of the host-callable surface.
- `.cargo/config.toml` — RISC-V link config (static, `+crt-static`,
  libc memory/string functions wrapped).
- `.github/workflows/guest.yml` — CI (builds the guest on Ubuntu runners).

## Licence

Apache-2.0 (see `LICENSE`).
