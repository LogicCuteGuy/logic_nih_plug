# Gain AAX Example

A simple gain plugin demonstrating AAX export using NIH-plug.

## Building

**Important**: Building AAX plugins requires the AAX SDK from Avid, which is only available to registered developers with an Avid developer account.

```bash
cargo build --release
```

## Format-Specific Requirements

### AAX Plugin Trait

The plugin implements the `AaxPlugin` trait to provide AAX-specific metadata:

```rust
impl AaxPlugin for GainAax {
    const AAX_MANUFACTURER_ID: [u8; 4] = *b"Mois";
    const AAX_PRODUCT_ID: i32 = 0x47414158; // "GAAX"
    const AAX_CATEGORY: AaxCategory = AaxCategory::EQ;
    const AAX_TYPE_IDS: &'static [AaxTypeId] = &[AaxTypeId::Native];
}
```

- **AAX_MANUFACTURER_ID**: 4-character manufacturer code (must be registered with Avid)
- **AAX_PRODUCT_ID**: Unique product ID (must be registered with Avid)
- **AAX_CATEGORY**: Plugin category for Pro Tools
- **AAX_TYPE_IDS**: Array of supported AAX types (Native, AudioSuite, etc.)

## Prerequisites

1. **Avid Developer Account**: Sign up at https://www.avid.com/alliance-partner-program
2. **AAX SDK**: Download from Avid developer portal
3. **Developer Certificate**: Obtain code signing certificate from Avid
4. **Manufacturer ID**: Register your manufacturer ID with Avid
5. **Product IDs**: Register product IDs for your plugins

## Bundling

To create a distributable AAX bundle:

```bash
cargo xtask bundle gain_aax --release
```

This will create `target/bundled/GainAax.aaxplugin/` bundle.

**Note**: The bundle must be code-signed with your Avid developer certificate before it will load in Pro Tools.

## Code Signing

AAX plugins must be signed with an Avid-issued certificate:

```bash
# Example signing command (certificate details vary)
codesign --sign "Developer ID Application: Your Name" \
         --timestamp \
         --options runtime \
         target/bundled/GainAax.aaxplugin
```

## Installation

Copy the `.aaxplugin` bundle to:
- **macOS**: `/Library/Application Support/Avid/Audio/Plug-Ins/`
- **Windows**: `C:\Program Files\Common Files\Avid\Audio\Plug-Ins\`

## Testing

Test the plugin in Pro Tools:
- Launch Pro Tools
- Create a new session
- Insert the plugin on a track
- Verify parameter automation works correctly

## Notes

- AAX is Pro Tools-specific
- Requires Avid developer account and SDK
- Requires code signing with Avid certificate
- Most restrictive licensing of all formats
- Supports Native, AudioSuite, and DSP processing
- Requires manufacturer and product ID registration
