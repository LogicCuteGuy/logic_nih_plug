# Gain LV2 Example

A simple gain plugin demonstrating LV2 export using NIH-plug.

## Building

```bash
cargo build --release
```

LV2 plugins can be built on all platforms but are primarily used on Linux.

## Format-Specific Requirements

### LV2 Plugin Trait

The plugin implements the `Lv2Plugin` trait to provide LV2-specific metadata:

```rust
impl Lv2Plugin for GainLv2 {
    const LV2_URI: &'static str = "https://github.com/robbert-vdh/nih-plug/examples/gain_lv2";
    const LV2_CATEGORY: Lv2Category = Lv2Category::AmplifierPlugin;
}
```

- **LV2_URI**: A unique URI identifying the plugin (should be a URL you control)
- **LV2_CATEGORY**: The plugin category for LV2 hosts

## Bundling

To create a distributable LV2 bundle:

```bash
cargo xtask bundle gain_lv2 --release
```

This will create `target/bundled/gain_lv2.lv2/` directory containing:
- `manifest.ttl` - LV2 manifest file
- `gain_lv2.ttl` - Plugin description
- `gain_lv2.so` (or `.dll`/`.dylib`) - Plugin binary

## Installation

Copy the `.lv2` bundle directory to:
- **Linux User**: `~/.lv2/`
- **Linux System**: `/usr/lib/lv2/` or `/usr/local/lib/lv2/`
- **macOS**: `~/Library/Audio/Plug-Ins/LV2/`
- **Windows**: `%APPDATA%\LV2\`

## Testing

Test the plugin in LV2-compatible hosts:
- **Ardour**
- **Qtractor**
- **Carla**
- **Mixbus**

Use `lv2ls` to list installed plugins:

```bash
lv2ls | grep gain_lv2
```

Use `lv2info` to inspect plugin metadata:

```bash
lv2info https://github.com/robbert-vdh/nih-plug/examples/gain_lv2
```

## Notes

- LV2 is an open-source plugin standard (ISC license)
- Uses port-based architecture
- Manifest files are generated automatically
- Supports extensive extension system
- Cross-platform but primarily used on Linux
