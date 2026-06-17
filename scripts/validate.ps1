#Requires -Version 5.1
<#
.SYNOPSIS
    Per-format plugin validator driver for this nih-plug project (Windows).

.DESCRIPTION
    This script lives at <project>\scripts\. It walks <project>\target\bundled\
    and dispatches each plugin binary to the right validator based on its file
    extension. Designed to run *after* `cargo xtask bundle ... --release` from
    within the project root.

.PARAMETER ProjectDir
    Override the project location. Defaults to <workspace>\logic_nih_plug.

.PARAMETER Filter
    Substring filter on artifact paths. Empty = all.

.PARAMETER Strictness
    pluginval strictness level (1-10). Default 5.

.PARAMETER IncludeGui
    Run GUI tests too (pluginval default omits them).

.PARAMETER SkipClap
    Skip CLAP validation.

.PARAMETER SkipVst3
    Skip VST3 validation.

.PARAMETER PluginValBin
    Override path to pluginval.exe. Defaults to "pluginval" on PATH.

.PARAMETER ClapValBin
    Override path to clap-validator.exe. Defaults to "clap-validator" on PATH.

.PARAMETER Vst3ValBin
    Override path to Steinberg validator.exe. Defaults to "validator" on PATH.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\validate.ps1
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\validate.ps1 -Filter gain -Strictness 5 -IncludeGui
#>
[CmdletBinding()]
param(
    [string]$ProjectDir = "",
    [string]$Filter = "",
    [int]$Strictness = 5,
    [switch]$IncludeGui,
    [switch]$SkipClap,
    [switch]$SkipVst3,
    [string]$PluginValBin = "pluginval",
    [string]$ClapValBin = "clap-validator",
    [string]$Vst3ValBin = "validator"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot   = (Resolve-Path (Join-Path $ScriptDir "..")).Path
if ($ProjectDir -eq "") {
    $ProjectDir = $RepoRoot
}
$BundleDir = Join-Path $ProjectDir "target/bundled"

function Write-Log  { param($m) Write-Host "[validate] $m" -ForegroundColor Cyan }
function Write-Fail { param($m) Write-Host "[validate] FAIL: $m" -ForegroundColor Red; exit 1 }

function Resolve-ValidatorBin {
    param([string]$Name)
    $localExe = Join-Path $ProjectDir "target/bin/$Name.exe"
    if (Test-Path $localExe) { return $localExe }
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Path }
    return $null
}

if (-not (Test-Path $ProjectDir)) {
    Write-Fail "no project at $ProjectDir. Pass -ProjectDir."
}
if (-not (Test-Path $BundleDir)) {
    Write-Fail "no bundle dir at $BundleDir. Run 'cargo xtask bundle ... --release' from $ProjectDir."
}

$artifacts = @()
Get-ChildItem -Path $BundleDir -Recurse -File -Include "*.clap","*.vst3" | ForEach-Object {
    if ($Filter -eq "" -or $_.FullName -like "*$Filter*") {
        $artifacts += $_
    }
}

if ($artifacts.Count -eq 0) {
    Write-Fail "no artifacts matched (filter='$Filter')"
}

Write-Log "found $($artifacts.Count) artifact(s) under $BundleDir"

foreach ($f in $artifacts) {
    switch ($f.Extension.ToLower()) {
        ".clap" {
            if ($SkipClap) { continue }
            $bin = Resolve-ValidatorBin $ClapValBin
            if (-not $bin) { Write-Fail "clap-validator not found. Run scripts/install_validators.sh or set -ClapValBin." }
            Write-Log "CLAP -> $($f.Name)"
            & $bin validate --only-failed $f.FullName
            if ($LASTEXITCODE -ne 0) { Write-Fail "clap-validator failed on $($f.Name)" }
        }
        ".vst3" {
            if ($SkipVst3) { continue }
            $sv = Resolve-ValidatorBin $Vst3ValBin
            if ($sv) {
                Write-Log "VST3 -> $($f.Name) (Steinberg validator)"
                & $sv $f.FullName
                if ($LASTEXITCODE -ne 0) { Write-Fail "Steinberg validator failed on $($f.Name)" }
                continue
            }
            $pv = Resolve-ValidatorBin $PluginValBin
            if (-not $pv) { Write-Fail "no VST3 validator found. Set -Vst3ValBin or install pluginval." }
            $guiFlag = if ($IncludeGui) { "" } else { "--skip-gui-tests" }
            Write-Log "VST3 -> $($f.Name) (pluginval strictness=$Strictness)"
            & $pv --validate $f.FullName --strictness-level $Strictness --timeout 600 $guiFlag
            if ($LASTEXITCODE -ne 0) { Write-Fail "pluginval failed on $($f.Name)" }
        }
    }
}

Write-Log "all checks passed ($($artifacts.Count) artifact(s))."
