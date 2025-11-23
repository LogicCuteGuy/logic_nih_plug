# Gain AU Example

A simple gain plugin demonstrating Audio Units (AU) export using NIH-plug.

## Building

```bash
cargo build --release
```

**Note**: AU plugins can only be built on macOS.

## Format-Specific Requirements

### AU Plugin Trait

The plugin implements the `AuPlugin` trait to provide AU-specific metadata:

```rust
impl AuPlugin for GainAu {
    const AU_TYPE: [u8; 4] = *b"aufx";
    const AU_SUBTYPE: [u8; 4] = *b"gain";
    const AU_MANUFACTURER: [u8; 4] = *b"Mois";
}
```

- **AU_TYPE**: 4-character code for plugin type (`aufx` = effect, `aumu` = instrument)
- **AU_SUBTYPE**: 4-character code uniquely identifying this plugin
- **AU_MANUFACTURER**: 4-character manufacturer code

## Bundling

To create a distributable AU component bundle:

```bash
cargo xtask bundle gain_au --release
```

This will create `target/bundled/GainAu.component/` bundle.

## Installation

Copy the `.component` bundle to:
- **User**: `~/Library/Audio/Plug-Ins/Components/`
- **System**: `/Library/Audio/Plug-Ins/Components/`

## Testing

Test the plugin in AU-compatible hosts:
- **Logic Pro**
- **GarageBand**
- **Ableton Live**
- **Reaper**

Use `auval` to validate the plugin:

```bash
auval -v aufx gain Mois
```

## Notes

- AU is macOS-only
- Uses pull-based audio rendering
- Supports preset save/load through AU preset format
- No additional licensing requirements (uses Apple frameworks)
