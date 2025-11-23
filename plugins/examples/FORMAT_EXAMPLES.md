# Multi-Format Plugin Examples

This directory contains example plugins demonstrating how to export NIH-plug plugins to various audio plugin formats.

## Available Examples

### Format-Specific Examples

Each of these examples demonstrates exporting to a single plugin format:

- **gain_vst2** - VST2 export example
- **gain_au** - Audio Units (AU) export example (macOS only)
- **gain_auv3** - Audio Units v3 (AUv3) export example (macOS/iOS only)
- **gain_lv2** - LV2 export example
- **gain_aax** - AAX export example (requires AAX SDK)

### Multi-Format Example

- **gain_multi_format** - Demonstrates exporting a single plugin to multiple formats simultaneously (VST2, VST3, AU, AUv3, LV2, CLAP)

## Building Examples

To build a specific example:

```bash
cargo build --release -p <example_name>
```

For example:

```bash
cargo build --release -p gain_vst2
cargo build --release -p gain_multi_format
```

## Bundling Examples

To create distributable plugin bundles:

```bash
cargo xtask bundle <example_name> --release
```

This will create platform-appropriate bundles in `target/bundled/`.

## Platform Considerations

### macOS-Only Formats

- **AU (Audio Units)**: Only available on macOS
- **AUv3 (Audio Units v3)**: Only available on macOS and iOS

These examples will only compile on macOS. On other platforms, the AU/AUv3-specific code is conditionally excluded.

### AAX Requirements

The AAX example requires:
- AAX SDK from Avid (requires developer account)
- Registered manufacturer ID and product ID
- Code signing certificate from Avid

Without these, the example will compile but cannot be loaded in Pro Tools.

## Format-Specific Traits

Each format requires implementing a format-specific trait in addition to the base `Plugin` trait:

### VST2

```rust
impl Vst2Plugin for MyPlugin {
    const VST2_UNIQUE_ID: i32 = 0x12345678;
    const VST2_CATEGORY: Vst2Category = Vst2Category::Effect;
}

nih_export_vst2!(MyPlugin);
```

### AU

```rust
impl AuPlugin for MyPlugin {
    const AU_TYPE: [u8; 4] = *b"aufx";
    const AU_SUBTYPE: [u8; 4] = *b"mypg";
    const AU_MANUFACTURER: [u8; 4] = *b"Manu";
}

nih_export_au!(MyPlugin);
```

### AUv3

```rust
impl Auv3Plugin for MyPlugin {
    const AUV3_COMPONENT_TYPE: [u8; 4] = *b"aufx";
    const AUV3_COMPONENT_SUBTYPE: [u8; 4] = *b"mypg";
    const AUV3_COMPONENT_MANUFACTURER: [u8; 4] = *b"Manu";
    const AUV3_TAGS: &'static [&'static str] = &["Effects"];
}

nih_export_auv3!(MyPlugin);
```

### LV2

```rust
impl Lv2Plugin for MyPlugin {
    const LV2_URI: &'static str = "https://example.com/plugins/myplugin";
    const LV2_CATEGORY: Lv2Category = Lv2Category::UtilityPlugin;
}

nih_export_lv2!(MyPlugin);
```

### AAX

```rust
impl AaxPlugin for MyPlugin {
    const AAX_MANUFACTURER_ID: [u8; 4] = *b"Manu";
    const AAX_PRODUCT_ID: i32 = 0x12345678;
    const AAX_CATEGORY: AaxCategory = AaxCategory::EQ;
    const AAX_TYPE_IDS: &'static [AaxTypeId] = &[AaxTypeId::Native];
}

nih_export_aax!(MyPlugin);
```

## Testing

Each example includes a README with format-specific testing instructions and recommended host applications.

## Documentation

For detailed information about each format, see:
- [Multi-Format Export Guide](../../../MULTI_FORMAT_EXPORT.md)
- Individual example README files
- Format-specific trait documentation in the NIH-plug API docs

## License

All examples are licensed under the ISC license, matching the NIH-plug framework.
