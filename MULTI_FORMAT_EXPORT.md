# Multi-Format Plugin Export Guide

This guide covers NIH-plug's support for exporting plugins in multiple audio plugin formats beyond VST3 and CLAP. The supported formats are:

- **VST2** - Steinberg's legacy Virtual Studio Technology format
- **AU** - Apple's Audio Units format for macOS
- **AUv3** - Audio Units version 3 for iOS and macOS
- **LV2** - Open-source plugin standard for Linux
- **AAX** - Avid Audio eXtension for Pro Tools

## Table of Contents

- [Quick Start](#quick-start)
- [VST2 Export](#vst2-export)
- [AU Export](#au-export)
- [AUv3 Export](#auv3-export)
- [LV2 Export](#lv2-export)
- [AAX Export](#aax-export)
- [Platform Requirements](#platform-requirements)
- [Licensing Considerations](#licensing-considerations)
- [Testing Your Plugins](#testing-your-plugins)
- [Known Limitations](#known-limitations)
- [Troubleshooting](#troubleshooting)

## Quick Start

To export your plugin in multiple formats, you need to:

1. Enable the desired format features in your `Cargo.toml`
2. Implement the format-specific trait for your plugin
3. Add the export macro to your plugin's `lib.rs`
4. Build and bundle your plugin using `cargo xtask bundle`

### Example: Multi-Format Plugin

```toml
# Cargo.toml
[dependencies]
nih_plug = { version = "*", features = ["vst2", "au", "lv2"] }
```

```rust
// lib.rs
use nih_plug::prelude::*;

struct MyPlugin {
    params: Arc<MyParams>,
}

impl Plugin for MyPlugin {
    // ... standard Plugin implementation
}

// VST2 support
impl Vst2Plugin for MyPlugin {
    const VST2_UNIQUE_ID: i32 = i32::from_be_bytes(*b"MyPl");
    const VST2_CATEGORY: Vst2Category = Vst2Category::Effect;
}

// AU support
#[cfg(target_os = "macos")]
impl AuPlugin for MyPlugin {
    const AU_TYPE: [u8; 4] = *b"aufx";
    const AU_SUBTYPE: [u8; 4] = *b"MyPl";
    const AU_MANUFACTURER: [u8; 4] = *b"Mfgr";
}

// LV2 support
impl Lv2Plugin for MyPlugin {
    const LV2_URI: &'static str = "http://example.org/plugins/my-plugin";
    const LV2_CATEGORY: Lv2Category = Lv2Category::DelayPlugin;
}

// Export macros
nih_export_vst3!(MyPlugin);
nih_export_clap!(MyPlugin);
nih_export_vst2!(MyPlugin);

#[cfg(target_os = "macos")]
nih_export_au!(MyPlugin);

nih_export_lv2!(MyPlugin);
```

## VST2 Export

VST2 is Steinberg's legacy plugin format, still widely used despite being officially deprecated.

### Cargo Features

```toml
[dependencies]
nih_plug = { version = "*", features = ["vst2"] }
```

### Implementation

Implement the `Vst2Plugin` trait:

```rust
use nih_plug::prelude::*;

impl Vst2Plugin for MyPlugin {
    /// Unique 32-bit plugin ID
    /// Generate using a 4-character code: i32::from_be_bytes(*b"MyPl")
    /// IMPORTANT: Never change this after release!
    const VST2_UNIQUE_ID: i32 = i32::from_be_bytes(*b"MyPl");
    
    /// Plugin category for host organization
    const VST2_CATEGORY: Vst2Category = Vst2Category::Effect;
}
```

### Available Categories

- `Vst2Category::Unknown` - Unknown or unspecified
- `Vst2Category::Effect` - Effect plugin
- `Vst2Category::Synth` - Synthesizer
- `Vst2Category::Analysis` - Analysis tool
- `Vst2Category::Mastering` - Mastering effect
- `Vst2Category::Spacializer` - Spatial effect
- `Vst2Category::RoomFx` - Room effect
- `Vst2Category::SurroundFx` - Surround effect
- `Vst2Category::Restoration` - Restoration tool
- `Vst2Category::OfflineProcess` - Offline processor
- `Vst2Category::Shell` - Shell plugin (container)
- `Vst2Category::Generator` - Generator

### Export Macro

```rust
nih_export_vst2!(MyPlugin);
```

### Platform-Specific Notes

- **Windows**: Generates `.dll` files
- **macOS**: Generates `.vst` bundle
- **Linux**: Generates `.so` files

### VST2 Limitations

- Deprecated format (Steinberg no longer licenses new VST2 plugins)
- Limited parameter automation compared to VST3
- No native support for polyphonic expression
- Maximum 16 MIDI channels

## AU Export

Audio Units is Apple's plugin format for macOS and iOS.

### Cargo Features

```toml
[dependencies]
nih_plug = { version = "*", features = ["au"] }
```

### Implementation

Implement the `AuPlugin` trait (macOS only):

```rust
use nih_plug::prelude::*;

#[cfg(target_os = "macos")]
impl AuPlugin for MyPlugin {
    /// AU type code (4 characters)
    /// Common values:
    /// - *b"aufx" for effects
    /// - *b"aumu" for instruments/synthesizers
    /// - *b"aumf" for music effects
    const AU_TYPE: [u8; 4] = *b"aufx";
    
    /// Unique subtype code (4 characters)
    /// This identifies your specific plugin
    const AU_SUBTYPE: [u8; 4] = *b"MyPl";
    
    /// Manufacturer code (4 characters)
    /// Use the same code across all your plugins
    const AU_MANUFACTURER: [u8; 4] = *b"Mfgr";
}
```

### Available AU Types

- `AuType::Effect` - Effect plugin (`aufx`)
- `AuType::MusicEffect` - Music effect (`aumf`)
- `AuType::Instrument` - Synthesizer (`aumu`)
- `AuType::Generator` - Generator (`augn`)
- `AuType::MidiProcessor` - MIDI processor (`aumi`)
- `AuType::OfflineEffect` - Offline effect (`auol`)
- `AuType::Panner` - Panner (`aupn`)
- `AuType::FormatConverter` - Format converter (`aufc`)
- `AuType::Output` - Output (`auou`)

### Export Macro

```rust
#[cfg(target_os = "macos")]
nih_export_au!(MyPlugin);
```

### Platform Requirements

- **macOS only** - AU is not available on other platforms
- Generates `.component` bundle
- Requires Component Manager registration

### AU-Specific Features

- Pull-based audio rendering model
- Property-based parameter system
- Native preset support
- Integrated with macOS audio system

## AUv3 Export

AUv3 is the modern Audio Units format using app extensions for iOS and macOS.

### Cargo Features

```toml
[dependencies]
nih_plug = { version = "*", features = ["auv3"] }
```

### Implementation

Implement the `Auv3Plugin` trait:

```rust
use nih_plug::prelude::*;

#[cfg(all(feature = "auv3", target_os = "macos"))]
impl Auv3Plugin for MyPlugin {
    /// Component type code (4 characters)
    const AUV3_COMPONENT_TYPE: [u8; 4] = *b"aufx";
    
    /// Component subtype code (4 characters)
    const AUV3_COMPONENT_SUBTYPE: [u8; 4] = *b"MyPl";
    
    /// Component manufacturer code (4 characters)
    const AUV3_COMPONENT_MANUFACTURER: [u8; 4] = *b"Mfgr";
    
    /// Tags for plugin categorization
    const AUV3_TAGS: &'static [Auv3Tag] = &[
        Auv3Tag::Effect,
        Auv3Tag::Delay,
    ];
}
```

### Available Tags

- `Auv3Tag::Effect` - Effect plugin
- `Auv3Tag::Synth` - Synthesizer
- `Auv3Tag::Delay` - Delay effect
- `Auv3Tag::Distortion` - Distortion
- `Auv3Tag::Dynamics` - Dynamics processing
- `Auv3Tag::EQ` - Equalizer
- `Auv3Tag::Filter` - Filter
- `Auv3Tag::Reverb` - Reverb
- `Auv3Tag::Modulation` - Modulation effects
- `Auv3Tag::PitchShift` - Pitch shifting
- `Auv3Tag::Spatial` - Spatial effects
- `Auv3Tag::Generator` - Generator
- `Auv3Tag::MIDIEffect` - MIDI effect
- `Auv3Tag::Mixer` - Mixer
- `Auv3Tag::Sampler` - Sampler
- `Auv3Tag::Utility` - Utility

### Export Macro

```rust
#[cfg(all(feature = "auv3", target_os = "macos"))]
nih_export_auv3!(MyPlugin);
```

### Platform Requirements

- **macOS 10.11+ or iOS 9.0+**
- Requires Xcode project with app extension target
- Requires proper Info.plist configuration
- Requires code signing with appropriate entitlements

### App Extension Setup

AUv3 plugins require additional setup beyond the Rust code:

1. **Create App Extension Target in Xcode**
   - Add a new "Audio Unit Extension" target to your Xcode project
   - Link against the AudioToolbox framework

2. **Configure Info.plist**
   ```xml
   <key>NSExtension</key>
   <dict>
       <key>NSExtensionPrincipalClass</key>
       <string>YourAudioUnitClass</string>
       <key>NSExtensionPointIdentifier</key>
       <string>com.apple.AudioUnit-UI</string>
   </dict>
   <key>AudioComponents</key>
   <array>
       <dict>
           <key>type</key>
           <string>aufx</string>
           <key>subtype</key>
           <string>MyPl</string>
           <key>manufacturer</key>
           <string>Mfgr</string>
           <key>name</key>
           <string>My Plugin</string>
           <key>version</key>
           <integer>1</integer>
       </dict>
   </array>
   ```

3. **Set Up Code Signing**
   - Configure appropriate entitlements
   - Sign with valid developer certificate

4. **Build the App Extension Bundle**
   - Build through Xcode or xcodebuild

### AUv3-Specific Features

- Sandboxed environment for security
- Inter-process communication
- View controller-based UI
- Modern iOS/macOS integration

## LV2 Export

LV2 is an open-source plugin standard primarily used on Linux.

### Cargo Features

```toml
[dependencies]
nih_plug = { version = "*", features = ["lv2"] }
```

### Implementation

Implement the `Lv2Plugin` trait:

```rust
use nih_plug::prelude::*;

impl Lv2Plugin for MyPlugin {
    /// LV2 URI - must be unique and should use a domain you control
    /// IMPORTANT: Never change this after release!
    const LV2_URI: &'static str = "http://example.org/plugins/my-plugin";
    
    /// Plugin category for host organization
    const LV2_CATEGORY: Lv2Category = Lv2Category::DelayPlugin;
}
```

### Available Categories

- `Lv2Category::Plugin` - Generic plugin
- `Lv2Category::DelayPlugin` - Delay effect
- `Lv2Category::DistortionPlugin` - Distortion
- `Lv2Category::DynamicsPlugin` - Dynamics processor
- `Lv2Category::EQPlugin` - Equalizer
- `Lv2Category::FilterPlugin` - Filter
- `Lv2Category::ModulatorPlugin` - Modulation effect
- `Lv2Category::ReverbPlugin` - Reverb
- `Lv2Category::SimulatorPlugin` - Simulator
- `Lv2Category::SpatialPlugin` - Spatial effect
- `Lv2Category::SpectralPlugin` - Spectral processor
- `Lv2Category::UtilityPlugin` - Utility
- `Lv2Category::AnalyserPlugin` - Analyzer
- `Lv2Category::ConverterPlugin` - Converter
- `Lv2Category::FunctionPlugin` - Function generator
- `Lv2Category::MixerPlugin` - Mixer
- `Lv2Category::InstrumentPlugin` - Instrument
- `Lv2Category::OscillatorPlugin` - Oscillator
- `Lv2Category::GeneratorPlugin` - Generator
- `Lv2Category::MIDIPlugin` - MIDI utility

### Export Macro

```rust
nih_export_lv2!(MyPlugin);
```

### Bundle Structure

LV2 plugins are distributed as bundles containing:

```
my-plugin.lv2/
├── manifest.ttl      # Bundle manifest
├── my-plugin.ttl     # Plugin description
└── my-plugin.so      # Plugin binary
```

The bundler automatically generates the `.ttl` files based on your plugin's metadata.

### Manifest Generation

The LV2 wrapper automatically generates valid RDF/Turtle manifest files:

```rust
// Generate manifest programmatically
let (manifest, plugin_ttl) = generate_lv2_bundle().unwrap();
```

### Platform Support

- **Linux**: Primary platform, full support
- **macOS**: Supported
- **Windows**: Supported but less common

### LV2-Specific Features

- Port-based architecture
- Extension system (state, worker, UI, etc.)
- URI-based identification
- RDF/Turtle metadata

## AAX Export

AAX is Avid's plugin format for Pro Tools.

### Cargo Features

```toml
[dependencies]
nih_plug = { version = "*", features = ["aax"] }
```

### Implementation

Implement the `AaxPlugin` trait:

```rust
use nih_plug::prelude::*;

impl AaxPlugin for MyPlugin {
    /// Manufacturer ID from Avid (4 characters)
    /// Must be obtained through Avid developer program
    const AAX_MANUFACTURER_ID: [u8; 4] = *b"Mfgr";
    
    /// Product ID - unique identifier for your plugin
    const AAX_PRODUCT_ID: i32 = 0x12345678;
    
    /// Plugin category
    const AAX_CATEGORY: AaxCategory = AaxCategory::Effect;
    
    /// Type IDs (Native, AudioSuite, etc.)
    const AAX_TYPE_IDS: &'static [AaxTypeId] = &[AaxTypeId::Native];
}
```

### Export Macro

```rust
nih_export_aax!(MyPlugin);
```

### AAX SDK Requirements

**IMPORTANT**: AAX support requires the proprietary AAX SDK from Avid.

1. **Obtain AAX SDK**
   - Register for Avid developer program
   - Sign developer agreement
   - Download AAX SDK

2. **Get Developer Credentials**
   - Manufacturer ID (4-character code)
   - Product IDs for your plugins
   - Code signing certificate

3. **Link AAX SDK**
   - Add AAX SDK to your build system
   - Link against AAX libraries
   - Implement AAX SDK integration

### Platform Requirements

- **Windows and macOS** - AAX is not available on Linux
- Requires AAX SDK (proprietary)
- Requires Avid developer account
- Requires code signing with Avid certificate
- Generates `.aaxplugin` bundle

### AAX-Specific Features

- Pro Tools integration
- Algorithmic delay compensation
- Chunk-based processing
- AAX-specific parameter system

### Current Implementation Status

The current AAX implementation provides the structure and trait definitions but requires full AAX SDK integration to function as a real AAX plugin. This is due to the proprietary nature of the AAX SDK.

## Platform Requirements

### Operating System Support

| Format | Windows | macOS | Linux | iOS |
|--------|---------|-------|-------|-----|
| VST2   | ✅      | ✅    | ✅    | ❌  |
| AU     | ❌      | ✅    | ❌    | ❌  |
| AUv3   | ❌      | ✅    | ❌    | ✅  |
| LV2    | ✅      | ✅    | ✅    | ❌  |
| AAX    | ✅      | ✅    | ❌    | ❌  |

### Build Requirements

#### VST2
- Rust stable toolchain
- No additional dependencies

#### AU
- macOS 10.7+
- Xcode Command Line Tools
- CoreAudio framework

#### AUv3
- macOS 10.11+ or iOS 9.0+
- Xcode with app extension support
- Code signing certificate
- AudioToolbox framework

#### LV2
- Rust stable toolchain
- No additional dependencies

#### AAX
- Windows or macOS
- AAX SDK from Avid
- Avid developer account
- Code signing certificate from Avid

## Licensing Considerations

### VST2

- **SDK License**: Steinberg VST2 SDK (permissive for distribution)
- **Status**: Deprecated by Steinberg, no new licenses issued
- **Distribution**: Existing VST2 plugins can still be distributed
- **Recommendation**: Use VST3 for new projects when possible

### AU / AUv3

- **SDK License**: Apple frameworks (no additional license required)
- **Status**: Active and supported by Apple
- **Distribution**: Free to distribute
- **Requirements**: macOS/iOS developer account for code signing

### LV2

- **SDK License**: ISC license (permissive)
- **Status**: Active open-source project
- **Distribution**: Free to distribute
- **Requirements**: None

### AAX

- **SDK License**: Proprietary Avid license
- **Status**: Active, requires developer agreement
- **Distribution**: Requires Avid developer account
- **Requirements**: 
  - Signed developer agreement with Avid
  - Annual developer program fee
  - Code signing certificate from Avid
  - Compliance with Avid's distribution terms

### NIH-plug Framework

- **License**: ISC license (permissive)
- **VST3 Bindings**: GPLv3 (affects VST3 plugins only)
- **Note**: VST3 plugins must comply with GPLv3 due to vst3-sys bindings

## Testing Your Plugins

### Validation Tools

#### VST2
- **pluginval** - Cross-platform VST2/VST3 validator
- **Reaper** - Good VST2 host for testing
- **FL Studio** - Windows VST2 testing

#### AU
- **auval** - Apple's AU validation tool (included with macOS)
  ```bash
  auval -v aufx MyPl Mfgr
  ```
- **Logic Pro** - Comprehensive AU testing
- **GarageBand** - Basic AU testing

#### AUv3
- **AUM** (iOS) - Popular AUv3 host for iOS
- **Logic Pro** (macOS) - Full AUv3 support
- **GarageBand** (iOS/macOS) - Basic AUv3 testing

#### LV2
- **lv2lint** - LV2 plugin validator
  ```bash
  lv2lint http://example.org/plugins/my-plugin
  ```
- **Ardour** - Professional LV2 host
- **Qtractor** - Linux LV2 testing
- **Carla** - Cross-platform LV2 host

#### AAX
- **Pro Tools** - Official AAX host
- **AAX SDK validation tools** - Included with AAX SDK

### Testing Checklist

For each format, verify:

- [ ] Plugin loads without errors
- [ ] All parameters are exposed correctly
- [ ] Parameter automation works
- [ ] Audio processing produces expected output
- [ ] MIDI events are received and processed
- [ ] Preset save/load works (where applicable)
- [ ] GUI displays correctly (if applicable)
- [ ] Plugin unloads cleanly
- [ ] No memory leaks
- [ ] No audio glitches or dropouts

### Automated Testing

Run the included test suite:

```bash
cargo test --features vst2,au,lv2
```

Property-based tests verify:
- Parameter mapping correctness
- Audio buffer routing
- MIDI event translation
- State serialization round-trips

## Known Limitations

### VST2

- **Deprecated Format**: No new VST2 licenses from Steinberg
- **Limited Automation**: Less sophisticated than VST3
- **No Polyphonic Expression**: Limited to traditional MIDI
- **16 MIDI Channels**: Hard limit in specification
- **No Native Sidechain**: Requires workarounds

### AU

- **macOS Only**: Not available on other platforms
- **Pull-Based Rendering**: Different from push-based formats
- **Component Manager**: Legacy registration system
- **Limited Linux Support**: None

### AUv3

- **App Extension Complexity**: Requires Xcode project setup
- **Code Signing Required**: Cannot test without certificate
- **Platform Specific**: macOS/iOS only
- **Sandboxing Restrictions**: Limited file system access
- **Additional Build Steps**: Beyond Rust compilation

### LV2

- **Manifest Generation**: Requires build-time generation
- **Port-Based Model**: Different from callback-based formats
- **Limited Windows Support**: Less common on Windows
- **Host Compatibility**: Varies by host implementation

### AAX

- **Proprietary SDK**: Requires Avid developer account
- **Code Signing**: Must be signed by Avid certificate
- **Platform Limited**: Windows and macOS only
- **Pro Tools Specific**: Limited host support
- **Annual Fees**: Developer program costs
- **Current Implementation**: Requires full SDK integration

### General Limitations

- **GUI Frameworks**: Format-specific GUI integration varies
- **Parameter Count**: Some formats have limits
- **Latency Reporting**: Format-specific implementations
- **Sidechain Support**: Not all formats support sidechaining
- **MIDI 2.0**: Not yet supported in any format

## Troubleshooting

### VST2 Issues

**Plugin doesn't load in host**
- Verify VST2_UNIQUE_ID is set correctly
- Check that the plugin binary is in the correct location
- Ensure the plugin exports the correct entry points

**Parameters not visible**
- Verify Plugin trait implementation
- Check parameter ID stability
- Ensure parameters are properly registered

### AU Issues

**auval fails**
- Check AU_TYPE, AU_SUBTYPE, and AU_MANUFACTURER codes
- Verify component registration
- Run auval with verbose output: `auval -v aufx MyPl Mfgr`

**Plugin not found by host**
- Verify .component bundle structure
- Check Info.plist configuration
- Clear AU cache: `killall -9 AudioComponentRegistrar`

### AUv3 Issues

**App extension won't load**
- Verify code signing certificate
- Check Info.plist configuration
- Ensure entitlements are correct
- Verify AudioComponents array in Info.plist

**Plugin not visible in host**
- Check component type/subtype/manufacturer codes
- Verify app extension is properly installed
- Restart host application

### LV2 Issues

**lv2lint errors**
- Check LV2_URI format (must be valid URI)
- Verify manifest.ttl syntax
- Ensure plugin.ttl is valid RDF/Turtle

**Plugin not found by host**
- Verify bundle structure (.lv2 directory)
- Check that manifest.ttl and plugin.ttl are present
- Ensure plugin binary has correct name
- Refresh LV2 cache in host

### AAX Issues

**Plugin won't load in Pro Tools**
- Verify AAX SDK integration
- Check code signing with Avid certificate
- Ensure manufacturer ID and product ID are correct
- Verify .aaxplugin bundle structure

### Build Issues

**Feature not enabled**
```
error: cannot find macro `nih_export_vst2` in this scope
```
Solution: Add the feature to Cargo.toml:
```toml
nih_plug = { version = "*", features = ["vst2"] }
```

**Platform mismatch**
```
error: AU is only available on macOS
```
Solution: Use conditional compilation:
```rust
#[cfg(target_os = "macos")]
nih_export_au!(MyPlugin);
```

**Missing SDK**
```
error: AAX SDK not found
```
Solution: Obtain AAX SDK from Avid and configure build system

### Getting Help

- Check the [NIH-plug documentation](https://nih-plug.robbertvanderhelm.nl/)
- Review example plugins in `plugins/examples/`
- Search existing GitHub issues
- Ask in the NIH-plug community

## Additional Resources

### Documentation

- [NIH-plug API Documentation](https://nih-plug.robbertvanderhelm.nl/)
- [VST2 SDK Documentation](https://www.steinberg.net/vst-sdk/) (archived)
- [Audio Units Programming Guide](https://developer.apple.com/documentation/audiounit)
- [LV2 Specification](http://lv2plug.in/ns/)
- [AAX SDK Documentation](https://www.avid.com/alliance-partner-program) (requires account)

### Example Plugins

See the `plugins/examples/` directory for working examples:
- `gain` - Basic plugin structure
- `gain_gui_*` - GUI integration examples
- `midi_inverter` - MIDI event handling
- `sine` - Audio generation

### Community

- [NIH-plug GitHub](https://github.com/robbert-vdh/nih-plug)
- [Rust Audio Discord](https://discord.gg/Qs2Zwtf9Gf)

---

*Last updated: 2024*
*NIH-plug version: 0.1.0*
