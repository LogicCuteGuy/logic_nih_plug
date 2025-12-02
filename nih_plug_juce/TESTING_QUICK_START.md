# Testing Quick Start

Quick reference for testing nih_plug_juce on your platform.

## Prerequisites Check

### All Platforms
```bash
# Verify Rust installation
rustc --version
cargo --version

# Verify CMake installation
cmake --version
```

### Linux
```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y \
    build-essential cmake pkg-config \
    libasound2-dev libx11-dev libxext-dev \
    libxrandr-dev libxinerama-dev libxcursor-dev \
    libfreetype6-dev libgl1-mesa-dev

# Verify compiler
gcc --version
g++ --version
```

### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install CMake (via Homebrew)
brew install cmake

# Verify compiler
clang --version
```

### Windows (PowerShell)
```powershell
# Verify Visual Studio installation
"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

# Or check with vswhere
vswhere -latest -property installationPath

# Verify CMake
cmake --version
```

## Quick Test Commands

### 1. Build Core Library
```bash
cargo build -p nih_plug_juce
```
**Expected:** Should complete without errors

### 2. Build Examples
```bash
cargo build -p juce_ffi_button
cargo build -p juce_ffi_drawing
cargo build -p juce_ffi_layout
```
**Expected:** All should build successfully

### 3. Run Tests
```bash
# Library tests
cargo test -p nih_plug_juce --lib

# Integration tests
cargo test -p nih_plug_juce --tests
```
**Expected:** Most tests pass (some may require message thread)

### 4. Run Benchmarks
```bash
cargo bench -p nih_plug_juce --bench ffi_benchmarks
```
**Expected:** Completes with performance metrics

## Common Issues

### Linux: X11 Display Not Available
```bash
# Use Xvfb for headless testing
xvfb-run cargo test -p nih_plug_juce
```

### macOS: Framework Not Found
```bash
# Ensure Xcode Command Line Tools installed
xcode-select --install
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

### Windows: MSVC Not Found
```powershell
# Run from Visual Studio Developer Command Prompt
# Or set environment:
"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
```

## Reporting Results

If you test on macOS or Windows, please report results by creating an issue with:

1. Platform information (OS version, compiler version)
2. Output of build commands
3. Output of test commands
4. Any errors encountered

See [CROSS_PLATFORM_TESTING.md](CROSS_PLATFORM_TESTING.md) for detailed testing procedures.
