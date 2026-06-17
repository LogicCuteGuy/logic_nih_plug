# Bundling & Distribution

`cargo xtask` is the bundler. The shim binary is at [xtask/src/main.rs](../xtask/src/main.rs)
and calls [logic_nih_plug_xtask](../logic_nih_plug_xtask/). The `.cargo/config.toml` alias
pins it to release mode so `serde` is only built once.

## Common commands

```shell
cargo xtask bundle gain --release                       # default target
cargo xtask bundle-universal -p gain --release          # macOS universal
cargo xtask bundle gain --release \
  --target x86_64-unknown-linux-gnu                     # cross-compile
cargo xtask known-packages                             # list bundled crates
```

Output: `target/bundled/`.

## Plugin formats

The bundler auto-detects which exports your crate has and creates the right
folder layout for the target OS.

| Format | Export macro | Built-in |
|---|---|---|
| VST3 | `nih_export_vst3!` | yes |
| CLAP | `nih_export_clap!` | yes |
| VST2 | `nih_export_vst2!` | needs `vst2-sys` + local SDK |
| AU | `nih_export_au!` | macOS only |
| AUv3 | `nih_export_auv3!` | macOS/iOS only |
| LV2 | `nih_export_lv2!` | optional |
| AAX | `nih_export_aax!` | needs Avid SDK + cert |

See [FORMAT_EXAMPLES.md](../plugins/examples/FORMAT_EXAMPLES.md) for per-format
crates and the multi-format example.

> Don't enable `vst2` and `aax` in the same plugin — they collide at the linker.

## `cargo-nih-plug` subcommand

[cargo_logic_nih_plug/](../cargo_logic_nih_plug/) wraps the same engine as a global
`cargo nih-plug` subcommand for use outside this repo.

```shell
cargo install --git https://github.com/robbert-vdh/nih-plug.git cargo-nih-plug
cargo nih-plug bundle my_plugin --release
```

For in-repo work, prefer the bundled `cargo xtask` — it's pinned to the right
`logic_nih_plug_xtask` version.

## CI

[.github/workflows/build.yml](../.github/workflows/build.yml) — Linux + Windows,
nightly toolchain, GUI/X11 dev packages on Linux. `target/` is not cached on
Windows (runner overflow).

Plugins to bundle are enumerated by `cargo xtask known-packages`, driven by
[bundler.toml](../bundler.toml). Add a plugin there when introducing it.
