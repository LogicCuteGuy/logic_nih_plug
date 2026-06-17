# Getting Started

Five-minute orientation. For the full framework reference see the upstream
docs site (https://nih-plug.robbertvanderhelm.nl/) and [README.md](../README.md).

## Build a single plugin

```shell
cargo xtask bundle gain --release
```

Output lands in `target/bundled/`. CI uses `cargo xtask known-packages` to
enumerate plugins — see [bundler.toml](../bundler.toml).

## Build everything

```shell
cargo build --workspace
```

Slow (every plugin compiles). Use the per-crate build for day-to-day work.

## Run the test suite

```shell
cargo test --locked --workspace --features "simd,standalone,zstd"
```

> Never run `cargo test --all-features` — `logic_nih_plug_iced` has mutually
> exclusive features. The `--features "simd,standalone,zstd"` set is what CI
> uses; see [.github/workflows/build.yml](../.github/workflows/build.yml).

## Cross-compile / per-target

```shell
cargo xtask bundle gain --release --target x86_64-unknown-linux-gnu
cargo xtask bundle-universal -p gain --release   # macOS universal
```

## Toolchain

| Plugin | Toolchain | Why |
|---|---|---|
| most plugins | stable | default |
| `crossover`, `diopser` | nightly | uses `std::simd` via the `simd` feature |

## Hard rules

From [AGENTS.md §5](../AGENTS.md):

1. `process()` is real-time. No allocations, no blocking, no `println!`.
2. `initialize()` is the only place that may allocate heavily.
3. Cross-thread = `Arc<AtomicF32>`, `parking_lot::Mutex` (try_lock), or crossbeam channels.
4. Params need stable `#[id = "…"]` and `#[persist = "…"]` attrs.
5. Use `nih_log!` / `nih_dbg!` from `logic_nih_plug::debug`, not `println!` / `dbg!`.

## Next steps

- [Plugin skeleton walkthrough →](plugin-skeleton.md)
- [DSP & GUI modules →](dsp-and-gui.md)
- [Bundling & distribution →](bundling.md)
