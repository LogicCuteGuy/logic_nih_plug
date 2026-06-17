#!/usr/bin/env bash
# Install audio-plugin validator CLIs into ./target/bin/ (project-local).
#
# The Rust project IS this directory (logic_nih_plug/). Validators are installed
# into <project>/target/bin so they live alongside the plugin bundles but don't
# pollute the build cache.
#
# What this fetches:
#   - clap-validator  (Rust)        via `cargo install --locked`
#   - pluginval       (C++ binary)  via GitHub release zip
#
# What this does NOT fetch (you provide the binary yourself, see scripts/README.md):
#   - Steinberg VST3 `validator`    (ships with the public VST3 SDK)
#   - Apple `auval`                 (ships with Xcode Command Line Tools)
#
# Idempotent: skips anything that's already runnable.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROJECT_DIR="${PROJECT_DIR:-${REPO_ROOT}}"
BIN_DIR="${PROJECT_DIR}/target/bin"
mkdir -p "${BIN_DIR}"

log() { printf '[install_validators] %s\n' "$*" >&2; }
have() { command -v "$1" >/dev/null 2>&1; }

# ---- clap-validator ----------------------------------------------------------
if [[ -x "${BIN_DIR}/clap-validator" ]] || [[ -x "${BIN_DIR}/clap-validator.exe" ]] || have clap-validator; then
  log "clap-validator: already present"
else
  log "clap-validator: cargo install from git (project at ${PROJECT_DIR})"
  # clap-validator is not published on crates.io; install from upstream git.
  # cargo install writes to ~/.cargo/bin by default; install into our local bin.
  cargo install --git https://github.com/free-audio/clap-validator --root "${PROJECT_DIR}/target"
fi

# ---- pluginval ---------------------------------------------------------------
# Latest release URL pattern is stable: pluginval-<OS>-<ARCH>.zip, with two
# capitalisation variants seen across releases. We try the new style first
# and fall back to the old.
case "$(uname -s)" in
  Linux*)  PV_OS=Linux  ;;
  Darwin*) PV_OS=macOS  ;;
  *) log "unsupported OS for pluginval prebuilt: $(uname -s). Install manually."; PV_OS= ;;
esac
case "$(uname -m)" in
  x86_64|amd64) PV_ARCH=x64 ;;
  aarch64|arm64) PV_ARCH=arm64 ;;
  *) PV_ARCH= ;;
esac

if [[ -n "${PV_OS}" && -n "${PV_ARCH}" ]]; then
  if [[ -x "${BIN_DIR}/pluginval" ]] || [[ -x "${BIN_DIR}/pluginval.exe" ]] || have pluginval; then
    log "pluginval: already present"
  else
    log "pluginval: resolving latest release"
    tmp="$(mktemp -d)"
    for asset in "pluginval_${PV_OS}_${PV_ARCH}.zip" "pluginval-${PV_OS}-${PV_ARCH}.zip"; do
      url="https://github.com/Tracktion/pluginval/releases/latest/download/${asset}"
      if curl -fsSL --fail-with-body "${url}" -o "${tmp}/pluginval.zip" 2>/dev/null; then
        log "pluginval: downloaded ${asset}"
        break
      fi
    done
    [[ -f "${tmp}/pluginval.zip" ]] || { log "pluginval: failed to download"; rm -rf "${tmp}"; exit 1; }
    unzip -q "${tmp}/pluginval.zip" -d "${tmp}/out"
    found="$(find "${tmp}/out" -maxdepth 3 -type f \( -name 'pluginval' -o -name 'pluginval.exe' \) | head -n1)"
    [[ -n "${found}" ]] || { log "pluginval: binary not found in archive"; exit 1; }
    install -m 0755 "${found}" "${BIN_DIR}/$(basename "${found}")"
    rm -rf "${tmp}"
  fi
else
  log "pluginval: no prebuilt for this platform; install manually from https://github.com/Tracktion/pluginval/releases"
fi

log "done. PATH for this run:"
log "  export PATH=\"${BIN_DIR}:\$PATH\""
