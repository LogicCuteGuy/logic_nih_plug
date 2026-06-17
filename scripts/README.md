# Plugin export validation

CI scripts that exercise every binary `cargo xtask bundle` produces, using the
right per-format validator for each. Mirrors the validator landscape discussed
in the AGENTS table and the per-format docs.

## Layout

```
<workspace>/
├── scripts/             <- this folder
│   ├── install_validators.sh
│   ├── validate.sh
│   ├── validate.ps1
│   └── README.md
├── target/bin/          <- validator CLIs land here (git-ignored)
└── logic_nih_plug/      <- the Rust project (where `cargo xtask` runs)
    └── target/bundled/  <- plugin bundles that get validated
```

## What you get

| Format | Validator | Installed by |
|---|---|---|
| CLAP   | [`clap-validator`](https://github.com/free-audio/clap-validator) | `install_validators.sh` (cargo install) |
| VST3   | [`pluginval`](https://github.com/Tracktion/pluginval) **or** Steinberg `validator` | `install_validators.sh` / manual |
| AU     | Apple `auval` (`/usr/bin/auval`) | Ships with Xcode CLT — manual |
| AAX    | `AAXValidator` | Avid SDK only — manual |
| LV2    | `lv2lint` | Manual |

## Quick start

```bash
# 1. install the freely-redistributable validators into <workspace>/target/bin/
./scripts/install_validators.sh
export PATH="$(pwd)/target/bin:$PATH"

# 2. build every plugin in bundler.toml
cd logic_nih_plug
cargo xtask bundle $(cargo xtask known-packages | xargs -I{} echo -p {}) --release

# 3. validate everything that was bundled
cd ..
./scripts/validate.sh
```

To validate a single plugin, filter on substring:

```bash
./scripts/validate.sh gain                 # any artifact whose path contains 'gain'
./scripts/validate.sh soft_vacuum --gui    # include GUI tests
./scripts/validate.sh --strictness 7       # crank pluginval
```

Windows:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\validate.ps1 -Filter gain -Strictness 5
```

## GUI testing

Most validators skip GUI work by default to stay headless-safe. To enable GUI
tests on Linux CI you need Xvfb (installed on the `nih-plug` Linux runners
already):

```bash
sudo apt-get install -y xvfb
xvfb-run -a ./scripts/validate.sh --gui
```

`--gui` on Linux is equivalent to dropping the `--skip-gui-tests` flag from
`pluginval`. clap-validator always exercises GUI tests; it has no flag for
that.

On Windows runners a display is available. On macOS GUI tests run natively.

## Steinberg VST3 validator

This is a CLI host that ships inside the
[public VST3 SDK](https://github.com/steinbergmedia/vst3_public_sdk).
You build it yourself once:

```bash
git clone https://github.com/steinbergmedia/vst3_public_sdk
cmake -S vst3_public_sdk -B vst3_public_sdk/build -DVST3_BUILD_VALIDATOR=ON
cmake --build vst3_public_sdk/build --target validator
export VST3_VALIDATOR="$PWD/vst3_public_sdk/build/bin/validator"
```

Once `VST3_VALIDATOR` is set, `validate.sh` prefers it over pluginval.

## Overriding the project location

If the Rust project isn't at `<workspace>/logic_nih_plug`, set `PROJECT_DIR`:

```bash
PROJECT_DIR=/path/to/nih-plug ./scripts/validate.sh
powershell -ExecutionPolicy Bypass -File scripts\validate.ps1 -ProjectDir C:\path\to\nih-plug
```

## Per-format gotchas (matches the AGENTS.md rules)

- `simd` builds need nightly Rust (CI uses `dtolnay/rust-toolchain@nightly`).
- AU/AUv3 are macOS-only — the validator will simply skip them on other OSes.
- AAX is gated behind Avid credentials — only run it if you actually ship AAX.
- Windows has tiny audio-thread stacks. Validator won't hit this directly, but
  your `process()` regressions will surface as `pluginval` failures at
  strictness ≥ 3.

## CI

`.github/workflows/validate.yml` runs `install_validators.sh && bundle && validate.sh`
on Linux/Windows/macOS runners and uploads the validator logs as artifacts.
