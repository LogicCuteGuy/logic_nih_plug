# Gain Multi-Format Example

A simple gain plugin demonstrating multi-format export using NIH-plug.

This example shows how a single plugin implementation can export to multiple plugin formats simultaneously: VST2, VST3, AU, AUv3, LV2, and CLAP.

## Building

```bash
cargo build --release
```

## Format-Specific Traits

The plugin implements format-specific traits for each supported format:

### VST2
```rust
impl Vst2Plugin for GainMultiFormat {
    const VST2_UNIQUE_ID: i32 = 0x474D4654; // "GMFT"
    const VST2_CATEGORY: Vst2Category = Vst2Category::Effect;
}
```

### VST3
```rust
impl Vst3Plugin for GainMultiFormat {
    const VST3_CLASS_ID: [u8; 16] = *b"GainMultiFormatP";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[...];
}
```

### AU
```rust
impl AuPlugin for GainMultiFormat {
    const AU_TYPE: [u8; 4] = *b"aufx";
    const AU_SUBTYPE: [u8; 4] = *b"gmft";
    const AU_MANUFACTURER: [u8; 4] = *b"Mois";
}
```

### AUv3
```rust
impl Auv3Plugin for GainMultiFormat {
    const AUV3_COMPONENT_TYPE: [u8; 4] = *b"aufx";
    const AUV3_COMPONENT_SUBTYPE: [u8; 4] = *b"gmft";
    const AUV3_COMPONENT_MANUFACTURER: [u8; 4] = *b"Mois";
    const AUV3_TAGS: &'static [&'static str] = &["Effects"];
}
```

### LV2
```rust
impl Lv2Plugin for GainMultiFormat {
    const LV2_URI: &'static str = "https://github.com/robbert-vdh/nih-plug/examples/gain_multi_format";
    const LV2_CATEGORY: Lv2Category = Lv2Category::AmplifierPlugin;
}
```

### CLAP
```rust
impl ClapPlugin for GainMultiFormat {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain-multi-format";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("...");
    const CLAP_FEATURES: &'static [ClapFeature] = &[...];
}
```

## Export Macros

All format export macros are called at the end of the file:

```rust
nih_export_vst2!(GainMultiFormat);
nih_export_vst3!(GainMultiFormat);
nih_export_au!(GainMultiFormat);
nih_export_auv3!(GainMultiFormat);
nih_export_lv2!(GainMultiFormat);
nih_export_clap!(GainMultiFormat);
```

## Bundling

The bundler will automatically detect all export macros and create bundles for each format:

```bash
cargo xtask bundle gain_multi_format --release
```

This will create:
- `GainMultiFormat.vst` (VST2)
- `GainMultiFormat.vst3` (VST3)
- `GainMultiFormat.component` (AU, macOS only)
- `GainMultiFormat.appex` (AUv3, macOS/iOS only)
- `gain_multi_format.lv2/` (LV2)
- `GainMultiFormat.clap` (CLAP)

## Platform Considerations

Some formats are platform-specific:
- **AU/AUv3**: macOS/iOS only
- **VST2/VST3/CLAP/LV2**: All platforms

The build system will automatically skip platform-specific formats on unsupported platforms.

## Testing

Test the plugin in various hosts:
- **VST2**: Reaper, FL Studio
- **VST3**: Most modern DAWs
- **AU**: Logic Pro, GarageBand (macOS)
- **AUv3**: Logic Pro, AUM (macOS/iOS)
- **LV2**: Ardour, Qtractor (Linux)
- **CLAP**: Bitwig Studio, Reaper

## Benefits of Multi-Format Export

1. **Maximum Compatibility**: Support users on all platforms and DAWs
2. **Single Codebase**: Maintain one implementation for all formats
3. **Consistent Behavior**: Same audio processing across all formats
4. **Easy Maintenance**: Update once, deploy everywhere

## Notes

- Each format has its own metadata requirements
- Format-specific IDs must be unique across all your plugins
- Some formats require additional setup (AAX SDK, code signing)
- The bundler handles format-specific bundle structures automatically
