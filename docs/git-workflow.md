# Git Workflow

This is a fork of [robbert-vdh/nih-plug](https://github.com/robbert-vdh/nih-plug)
with JUCE-ported modules added. Two remotes, one default branch (`main`).

## Remotes

| Name | URL | Role |
|---|---|---|
| `origin` | `https://github.com/LogicCuteGuy/logic_nih_plug.git` | this fork — push here |
| `upstream` | `https://github.com/robbert-vdh/nih-plug.git` | sync only, do not push |

Verify with `git remote -v`.

## Branches

- `main` — default. Upstream uses `master`; this fork tracks `main`.
- CI builds any branch on push (`build.yml: push: branches: ['**']`) and
  every PR that targets `master` upstream — fork PRs target `main`.
- No `develop` / long-lived feature branches.

## Pull from upstream

```shell
git fetch upstream
git merge upstream/master   # or: git rebase upstream/master
git push origin main
```

Resolve conflicts case-by-case. The JUCE-ported crates are fork-only;
expect conflict-free merges in `src/`, `plugins/`, and CI workflows.

## Daily cycle

```shell
# 1. Sync
git pull --rebase origin main

# 2. Work in a topic branch if it's bigger than a one-shot fix
git switch -c topic/short-name
# ...edit, build, test...
cargo xtask bundle gain --release            # smoke build
cargo test --locked -p gain                  # focused tests
cargo test --locked --workspace --features "simd,standalone,zstd"  # full

# 3. Push
git push origin topic/short-name
# Open PR against origin/main (or upstream/master for upstream-bound changes)
```

## Commit style

No enforced convention in this repo. Keep commits focused; `CHANGELOG.md` is
**not** updated per-commit — it tracks breaking changes per day.

## CI gates (`.github/workflows/`)

| Workflow | Trigger | What it does |
|---|---|---|
| `build.yml` | push (any branch), PR to `master`, tags, manual | builds every package in `bundler.toml` on Ubuntu/macOS/Windows, uploads `nih-plugs-<date>-<runner>.zip` |
| `test.yml` | push, PR to `master` | `cargo test --locked --workspace --features "simd,standalone,zstd"` + `cargo build --no-default-features` |
| `docs.yml` | push | builds and publishes rustdoc to GitHub Pages |
| `juce_modules.yml` | — | JUCE module CI (added by this fork) |

Pushing to `main` runs `build.yml` and `docs.yml`. Tests do **not** run on
push to `main` here (they only run on upstream PRs against `master`); run the
test command locally before pushing:

```shell
cargo test --locked --workspace --features "simd,standalone,zstd"
```

## Toolchain

CI pins `dtolnay/rust-toolchain@nightly` because the `simd` feature needs
nightly `std::simd`. `rust-toolchain` is not committed, so local builds work on
stable unless you opt into `simd`:

```shell
cargo +nightly build -p diopser --features simd
```

## Release / publishing

There is no release pipeline for this fork. The upstream site
https://nih-plug.robbertvanderhelm.nl/ is built from `master`. If you cut a
fork release, bump `Cargo.toml` `version` and `git tag` per crate; the
`cargo xtask bundle` workflow tags the archive by date.
