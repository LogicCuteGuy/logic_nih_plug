# Gain VST2 Example

A simple gain plugin demonstrating VST2 export using NIH-plug.

## Building

```bash
cargo build --release
```

## Format-Specific Requirements

### VST2 Plugin Trait

The plugin implements the `Vst2Plugin` trait to provide VST2-specific metadata:

```rust
impl Vst2Plugin for GainVst2 {
    const VST2_UNIQUE_ID: i32 = 0x47564532; // "GVE2"
    const VST2_CATEGORY: Vst2Category = Vst2Category::Effect;
}
```

- **VST2_UNIQUE_ID**: A unique 32-bit identifier for the plugin. This should be unique across all VST2 plugins.
- **VST2_CATEGORY**: The plugin category (Effect, Synth, etc.)

## Bundling

To create a distributable VST2 bundle:

```bash
cargo xtask bundle gain_vst2 --release
```

This will create:
- **macOS**: `target/bundled/GainVst2.vst/` bundle
- **Windows**: `target/bundled/GainVst2.dll`
- **Linux**: `target/bundled/GainVst2.so`

## Testing

Test the plugin in VST2-compatible hosts:
- **Reaper** (all platforms)
- **FL Studio** (Windows)
- **Ableton Live** (all platforms, older versions)

## Notes

- VST2 is a deprecated format but still widely used
- The VST2 SDK license allows free distribution of VST2 plugins
- VST2 uses normalized parameter values (0.0-1.0)
