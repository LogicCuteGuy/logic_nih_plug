#!/usr/bin/env bash
# Per-format export validator driver for this nih-plug project.
#
# This script lives at <project>/scripts/. It walks <project>/target/bundled/
# and dispatches each plugin binary to the right validator(s) based on its
# file extension. Designed to run *after* `cargo xtask bundle ... --release`
# from within the project root.
#
# Usage:
#   scripts/validate.sh                  # validate everything in target/bundled
#   scripts/validate.sh gain             # only artifacts whose path contains 'gain'
#   scripts/validate.sh --strictness 5   # bump pluginval strictness (default 5)
#   scripts/validate.sh --gui            # include GUI tests on Linux (needs Xvfb)
#   scripts/validate.sh --no-clap        # skip CLAP
#   scripts/validate.sh --no-vst3        # skip VST3
#   scripts/validate.sh --no-au          # skip AU
#
# Env:
#   PROJECT_DIR       override project location (default: this script's parent)
#   PLUGINVAL_BIN     override pluginval binary
#   CLAPVAL_BIN       override clap-validator binary
#   VST3_VALIDATOR    path to Steinberg VST3 validator (preferred over pluginval)
#
# Exit code: 0 on full pass, non-zero on first failure (set -e).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROJECT_DIR="${PROJECT_DIR:-${REPO_ROOT}}"
BUNDLE_DIR="${PROJECT_DIR}/target/bundled"

STRICTNESS=5
INCLUDE_GUI=0
SKIP_CLAP=0
SKIP_VST3=0
SKIP_AU=0
FILTER=""
PLUGINVAL_BIN="${PLUGINVAL_BIN:-pluginval}"
CLAPVAL_BIN="${CLAPVAL_BIN:-clap-validator}"
VST3VAL_BIN="${VST3_VALIDATOR:-validator}"

usage() {
  sed -n '2,16p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --strictness) STRICTNESS="$2"; shift 2 ;;
    --gui) INCLUDE_GUI=1; shift ;;
    --no-clap) SKIP_CLAP=1; shift ;;
    --no-vst3) SKIP_VST3=1; shift ;;
    --no-au)  SKIP_AU=1; shift ;;
    --clap-validator) CLAPVAL_BIN="$2"; shift 2 ;;
    --pluginval) PLUGINVAL_BIN="$2"; shift 2 ;;
    --vst3-validator) VST3VAL_BIN="$2"; shift 2 ;;
    -*) echo "unknown flag: $1" >&2; usage; exit 2 ;;
    *)  FILTER="$1"; shift ;;
  esac
done

# Resolve a binary: prefer local install at <project>/target/bin/, then PATH.
resolve_bin() {
  local name="$1"
  if [[ -x "${PROJECT_DIR}/target/bin/${name}" ]]; then
    echo "${PROJECT_DIR}/target/bin/${name}"
    return 0
  fi
  if [[ -x "${PROJECT_DIR}/target/bin/${name}.exe" ]]; then
    echo "${PROJECT_DIR}/target/bin/${name}.exe"
    return 0
  fi
  if command -v "${name}" >/dev/null 2>&1; then
    command -v "${name}"
    return 0
  fi
  return 1
}

# Linux GUI runners: wrap the call in xvfb-run if available and requested.
maybe_xvfb() {
  if [[ "${INCLUDE_GUI}" -eq 1 ]] && command -v xvfb-run >/dev/null 2>&1; then
    echo "xvfb-run -a"
  fi
}

log()   { printf '\n[validate] %s\n' "$*" >&2; }
fail()  { printf '\n[validate] FAIL: %s\n' "$*" >&2; exit 1; }

[[ -d "${PROJECT_DIR}" ]] || fail "no project at ${PROJECT_DIR} — set PROJECT_DIR"
[[ -d "${BUNDLE_DIR}" ]] || fail "no bundle dir at ${BUNDLE_DIR} — run 'cargo xtask bundle ... --release' from ${PROJECT_DIR}"

shopt -s nullglob
ARTIFACTS=()
for f in "${BUNDLE_DIR}"/**/*; do
  case "${f}" in
    *.clap|*.vst3|*.component) ARTIFACTS+=("${f}") ;;
  esac
done
shopt -u nullglob

if [[ -n "${FILTER}" ]]; then
  filtered=()
  for a in "${ARTIFACTS[@]}"; do
    [[ "${a}" == *"${FILTER}"* ]] && filtered+=("${a}")
  done
  ARTIFACTS=("${filtered[@]}")
fi

[[ ${#ARTIFACTS[@]} -gt 0 ]] || fail "no artifacts matched (filter='${FILTER}')"

log "found ${#ARTIFACTS[@]} artifact(s) under ${BUNDLE_DIR}"

# ---- CLAP --------------------------------------------------------------------
run_clap() {
  local f="$1"
  [[ "${SKIP_CLAP}" -eq 0 ]] || return 0
  local bin
  bin="$(resolve_bin "${CLAPVAL_BIN}")" || fail "clap-validator not found. Run scripts/install_validators.sh or set CLAPVAL_BIN."
  log "CLAP  -> $(basename "${f}")"
  "${bin}" validate --only-failed "${f}"
}

# ---- VST3 --------------------------------------------------------------------
run_vst3() {
  local f="$1"
  [[ "${SKIP_VST3}" -eq 0 ]] || return 0
  local bin pv
  if bin="$(resolve_bin "${VST3VAL_BIN}")" 2>/dev/null; then
    log "VST3  -> $(basename "${f}")  (Steinberg validator)"
    "${bin}" "${f}"
    return
  fi
  if pv="$(resolve_bin "${PLUGINVAL_BIN}")" 2>/dev/null; then
    local gui_flag=""
    [[ "${INCLUDE_GUI}" -eq 0 ]] && gui_flag="--skip-gui-tests"
    local xvfb
    xvfb="$(maybe_xvfb)"
    log "VST3  -> $(basename "${f}")  (pluginval strictness=${STRICTNESS})"
    # shellcheck disable=SC2086
    ${xvfb} "${pv}" --validate "${f}" --strictness-level "${STRICTNESS}" --timeout 600 ${gui_flag}
    return
  fi
  fail "no VST3 validator found. Set VST3_VALIDATOR (Steinberg) or install pluginval."
}

# ---- AU ----------------------------------------------------------------------
run_au() {
  local f="$1"
  [[ "${SKIP_AU}" -eq 0 ]] || return 0
  command -v auval >/dev/null 2>&1 || { log "auval: not on PATH (macOS only) — skipping"; return 0; }
  log "AU    -> $(basename "${f}")"
  # auval needs the four-char codes. Discover them from Info.plist when we can.
  local plist="${f}/Contents/Info.plist"
  if [[ -f "${plist}" ]]; then
    local type sub manuf
    type="$(/usr/libexec/PlistBuddy -c 'Print :AudioComponents:0:type'       "${plist}" 2>/dev/null || echo '')"
    sub="$( /usr/libexec/PlistBuddy -c 'Print :AudioComponents:0:subtype'    "${plist}" 2>/dev/null || echo '')"
    manuf="$(/usr/libexec/PlistBuddy -c 'Print :AudioComponents:0:manufacturer' "${plist}" 2>/dev/null || echo '')"
    if [[ -n "${type}${sub}${manuf}" ]]; then
      auval -v "${type}" "${sub}" "${manuf}" -w
      return
    fi
  fi
  log "AU: could not read four-char codes from ${plist}; running discovery"
  auval -a | grep -i "$(basename "${f%.*}")" || true
}

# ---- dispatch ---------------------------------------------------------------
for f in "${ARTIFACTS[@]}"; do
  case "${f}" in
    *.clap)      run_clap "${f}" ;;
    *.vst3)      run_vst3 "${f}" ;;
    *.component) run_au   "${f}" ;;
  esac
done

log "all checks passed (${#ARTIFACTS[@]} artifact(s))."
