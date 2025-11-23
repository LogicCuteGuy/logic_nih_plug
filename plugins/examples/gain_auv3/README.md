# Gain AUv3 Example

A simple gain plugin demonstrating Audio Units v3 (AUv3) export using NIH-plug.

## Building

```bash
cargo build --release
```

**Note**: AUv3 plugins can only be built on macOS and iOS.

## Format-Specific Requirements

### AUv3 Plugin Trait

The plugin implements the `Auv3Plugin` trait to provide AUv3-specific metadata:

```rust
impl Auv3Plugin for GainAuv3 {
    const AUV3_COMPONENT_TYPE: [u8; 4] = *b"aufx";
    const AUV3_COMPONENT_SUBTYPE: [u8; 4] = *b"gain";
    const AUV3_COMPONENT_MANUFACTURER: [u8; 4] = *b"Mois";
    const AUV3_TAGS: &'static [&'static str] = &["Effects"];
}
```

- **AUV3_COMPONENT_TYPE**: 4-character code for plugin type
- **AUV3_COMPONENT_SUBTYPE**: 4-character code uniquely identifying this plugin
- **AUV3_COMPONENT_MANUFACTURER**: 4-character manufacturer code
- **AUV3_TAGS**: Array of category tags for the App Store

## Bundling

To create a distributable AUv3 app extension:

```bash
cargo xtask bundle gain_auv3 --release
```

This will create an app extension bundle suitable for distribution.

## Installation

AUv3 plugins are distributed as app extensions and require:
1. An Xcode project with app extension target
2. Code signing with a valid Apple Developer certificate
3. Distribution through the App Store or direct installation

## Testing

Test the plugin in AUv3-compatible hosts:
- **Logic Pro** (macOS)
- **GarageBand** (macOS/iOS)
- **AUM** (iOS)
- **Cubasis** (iOS)

## Notes

- AUv3 is the modern replacement for AU
- Supports both macOS and iOS
- Runs in a sandboxed app extension
- Requires Xcode for proper app extension setup
- Uses inter-process communication
- No additional licensing requirements
