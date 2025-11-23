# Design Document

## Overview

This design extends NIH-plug's plugin export system to support five additional audio plugin formats: VST2, AU (Audio Units), AUv3 (Audio Units v3), LV2, and AAX. The design follows the existing architecture pattern established by the VST3 and CLAP wrappers, where each format is implemented as:

1. A wrapper module that translates between the format's API and NIH-plug's Plugin trait
2. An export macro that generates the format-specific entry points
3. A format-specific plugin trait for metadata
4. Optional cargo features for conditional compilation

The implementation will maintain NIH-plug's philosophy of minimal boilerplate while providing format-specific customization where necessary.

## Architecture

### High-Level Structure

```
src/
├── wrapper/
│   ├── vst2/          # New: VST2 wrapper
│   ├── au/            # New: AU wrapper  
│   ├── auv3/          # New: AUv3 wrapper
│   ├── lv2/           # New: LV2 wrapper
│   ├── aax/           # New: AAX wrapper
│   ├── vst3/          # Existing
│   ├── clap/          # Existing
│   └── standalone/    # Existing
├── plugin/
│   ├── vst2.rs        # New: VST2 plugin trait
│   ├── au.rs          # New: AU plugin trait
│   ├── auv3.rs        # New: AUv3 plugin trait
│   ├── lv2.rs         # New: LV2 plugin trait
│   ├── aax.rs         # New: AAX plugin trait
│   ├── vst3.rs        # Existing
│   └── clap.rs        # Existing
```

### Wrapper Architecture Pattern

Each wrapper follows this structure (using VST2 as example):

```
wrapper/vst2/
├── mod.rs           # Export macro definition
├── wrapper.rs       # Main wrapper struct implementing format API
├── context.rs       # ProcessContext/InitContext implementations
├── descriptor.rs    # Plugin metadata handling
└── util.rs          # Helper functions
```

## Components and Interfaces

### 1. VST2 Wrapper

**Dependencies**: `vst2-sys` or similar VST2 bindings crate

**Key Components**:
- `Vst2Wrapper<P: Plugin>`: Main wrapper struct
- `nih_export_vst2!()` macro: Generates VST2 entry points
- `Vst2Plugin` trait: Format-specific metadata

**VST2-Specific Considerations**:
- VST2 uses a different parameter model (0.0-1.0 normalized values)
- Supports both effect and instrument plugins
- Requires unique plugin ID (32-bit integer)
- Editor integration via platform-specific window handles
- Deprecated format but still widely used

### 2. AU Wrapper

**Dependencies**: `coreaudio-sys`, custom AU bindings

**Key Components**:
- `AuWrapper<P: Plugin>`: Main wrapper struct
- `nih_export_au!()` macro: Generates AU component entry points
- `AuPlugin` trait: AU-specific metadata (type, subtype, manufacturer)

**AU-Specific Considerations**:
- macOS-only format
- Pull-based audio rendering model (different from push-based)
- Requires 4-character codes for type, subtype, and manufacturer
- Component Manager registration
- Property-based parameter system
- Preset management through AU preset format

### 3. AUv3 Wrapper

**Dependencies**: `coreaudio-sys`, iOS/macOS frameworks

**Key Components**:
- `Auv3Wrapper<P: Plugin>`: Main wrapper struct
- `nih_export_auv3!()` macro: Generates AUv3 app extension
- `Auv3Plugin` trait: AUv3-specific metadata

**AUv3-Specific Considerations**:
- iOS and macOS support
- App extension architecture
- View controller-based UI
- Sandboxed environment
- Inter-process communication
- Modern replacement for AU
- Requires Xcode project setup for app extension

### 4. LV2 Wrapper

**Dependencies**: `lv2-sys` or similar LV2 bindings

**Key Components**:
- `Lv2Wrapper<P: Plugin>`: Main wrapper struct
- `nih_export_lv2!()` macro: Generates LV2 descriptor and manifest
- `Lv2Plugin` trait: LV2-specific metadata (URI, category)

**LV2-Specific Considerations**:
- Primarily Linux, but cross-platform
- Port-based architecture (audio, control, MIDI)
- RDF/Turtle manifest files (manifest.ttl, plugin.ttl)
- URI-based identification
- Extension system (state, worker, UI, etc.)
- Requires generating manifest files at build time

### 5. AAX Wrapper

**Dependencies**: AAX SDK (proprietary, requires Avid developer account)

**Key Components**:
- `AaxWrapper<P: Plugin>`: Main wrapper struct
- `nih_export_aax!()` macro: Generates AAX entry points
- `AaxPlugin` trait: AAX-specific metadata

**AAX-Specific Considerations**:
- Requires AAX SDK (not publicly available)
- Requires signing with Avid developer certificate
- Pro Tools-specific format
- Algorithmic delay compensation
- Chunk-based processing
- Requires manufacturer ID and product ID from Avid
- Most restrictive licensing of all formats

## Data Models

### Format-Specific Plugin Traits

Each format requires a trait for format-specific metadata:

```rust
// VST2
pub trait Vst2Plugin: Plugin {
    const VST2_UNIQUE_ID: i32;
    const VST2_CATEGORY: Vst2Category;
}

// AU
pub trait AuPlugin: Plugin {
    const AU_TYPE: [u8; 4];           // e.g., b"aufx" for effect
    const AU_SUBTYPE: [u8; 4];        // Unique subtype code
    const AU_MANUFACTURER: [u8; 4];   // Manufacturer code
}

// AUv3
pub trait Auv3Plugin: Plugin {
    const AUV3_COMPONENT_TYPE: [u8; 4];
    const AUV3_COMPONENT_SUBTYPE: [u8; 4];
    const AUV3_COMPONENT_MANUFACTURER: [u8; 4];
    const AUV3_TAGS: &'static [&'static str];
}

// LV2
pub trait Lv2Plugin: Plugin {
    const LV2_URI: &'static str;      // e.g., "http://example.org/plugins/myplugin"
    const LV2_CATEGORY: Lv2Category;
}

// AAX
pub trait AaxPlugin: Plugin {
    const AAX_MANUFACTURER_ID: [u8; 4];
    const AAX_PRODUCT_ID: i32;
    const AAX_CATEGORY: AaxCategory;
    const AAX_TYPE_IDS: &'static [AaxTypeId];  // Native, AudioSuite, etc.
}
```

### Wrapper State Management

Each wrapper maintains:
- Reference to Plugin instance
- Parameter value cache for format-specific access patterns
- Audio buffer management for format's buffer model
- MIDI event translation buffers
- Editor state (if applicable)

### Parameter Mapping

Common parameter translation layer:
- NIH-plug normalized values (format-agnostic) ↔ Format-specific values
- Parameter change notifications
- Automation recording/playback
- Parameter grouping/hierarchy where supported

## Error Handling

### Initialization Errors

- Plugin fails to initialize: Return error code to host
- Invalid audio configuration: Reject and request alternative
- Missing required metadata: Compile-time error via trait bounds

### Runtime Errors

- Audio processing errors: Log and return silence
- Parameter out of range: Clamp to valid range
- MIDI parsing errors: Skip invalid events
- Editor errors: Gracefully degrade to no-GUI mode

### Build-Time Errors

- Missing format-specific trait implementation: Compile error
- Invalid metadata values: Compile error where possible
- Missing SDK dependencies: Clear error message with instructions

## Testing Strategy

### Unit Testing

Unit tests will verify:
- Parameter value conversions between NIH-plug and format-specific ranges
- MIDI event translation correctness
- Buffer management and audio routing
- Metadata generation for each format

### Property-Based Testing

Property-based tests will be written using the `proptest` crate to verify:
- Parameter conversion round-trips
- Audio buffer handling across different sizes
- MIDI event translation preserves semantics
- State serialization/deserialization

### Integration Testing

Integration tests will:
- Load plugins in format-specific validator tools
- Verify parameter automation
- Test preset save/load
- Verify audio processing produces expected output
- Test GUI integration where applicable

### Manual Testing

Manual testing in actual DAWs:
- VST2: Reaper, FL Studio
- AU: Logic Pro, GarageBand
- AUv3: AUM (iOS), Logic Pro (macOS)
- LV2: Ardour, Qtractor
- AAX: Pro Tools

## Implementation Notes

### Cargo Features

```toml
[features]
default = ["vst3"]
vst2 = ["dep:vst2-sys"]
au = ["dep:coreaudio-sys"]
auv3 = ["dep:coreaudio-sys", "au"]
lv2 = ["dep:lv2-sys"]
aax = ["dep:aax-sdk"]
```

### Platform Restrictions

- AU/AUv3: macOS/iOS only (enforced via `#[cfg(target_os = "macos")]`)
- AAX: Requires AAX SDK and developer account
- LV2: All platforms, but primarily Linux
- VST2: All platforms, but deprecated

### Licensing Considerations

- VST2: Steinberg SDK license (permissive for distribution)
- AU/AUv3: Apple frameworks (no additional license)
- LV2: ISC license (permissive)
- AAX: Requires Avid developer agreement and signing
- VST3: GPLv3 via vst3-sys (existing)

### Bundler Integration

The `nih_plug_xtask` bundler will be extended to:
1. Detect format export macros via source code parsing
2. Generate format-specific bundle structures
3. Copy binaries to correct locations
4. Generate manifest files (LV2)
5. Code sign where required (AAX, macOS)



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

After reviewing all acceptance criteria, I've identified the following testable correctness properties. Many criteria related to documentation quality, compile-time trait requirements, and GUI integration are not suitable for property-based testing.

### Parameter Mapping Properties

Property 1: Parameter exposure completeness
*For any* plugin with a set of parameters, when exported to any format (VST2, AU, AUv3, LV2, AAX), all parameters defined in the Plugin trait should be exposed to the host
**Validates: Requirements 1.2, 2.2, 3.2, 4.2, 5.2**

Property 2: Parameter automation updates
*For any* parameter and any format (VST2, AU, AUv3, LV2, AAX), when the host automates that parameter, the corresponding NIH-plug parameter value should be updated correctly
**Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**

Property 3: Parameter change notifications
*For any* parameter value change in the plugin, the system should notify the host according to the format's parameter change notification mechanism
**Validates: Requirements 10.6**

### Audio Processing Properties

Property 4: Audio buffer routing
*For any* audio buffer and any format (VST2, AU, AUv3, LV2, AAX), audio data should correctly flow from the host through the Plugin::process() method and back to the host
**Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.3**

### MIDI Translation Properties

Property 5: MIDI event translation correctness
*For any* valid MIDI event and any format (VST2, AU, LV2, AAX), the event should be correctly translated to NIH-plug's event format preserving all semantic information
**Validates: Requirements 1.4, 2.4, 4.4, 5.4**

### State Management Properties

Property 6: Preset round-trip consistency
*For any* plugin state, saving and loading through a format's preset mechanism (AU, LV2) should produce an equivalent state
**Validates: Requirements 2.5, 4.5**

### Build System Properties

Property 7: Export macro availability
*For any* format feature flag (vst2, au, auv3, lv2, aax), when enabled, the corresponding export macro should be available for use
**Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**

Property 8: Dependency isolation
*For any* format feature flag, when disabled, the system should not include that format's dependencies in the build
**Validates: Requirements 6.6**

Property 9: Valid plugin binary generation
*For any* plugin using an export macro (vst2, au, auv3, lv2, aax), the build system should generate a valid plugin binary with correct entry points for that format
**Validates: Requirements 1.1, 2.1, 3.1, 4.1, 5.1**

### Bundler Properties

Property 10: Format detection accuracy
*For any* plugin source file, the bundler should correctly detect all `nih_export_<format>!()` macros present in the code
**Validates: Requirements 7.1**

Property 11: Bundle structure correctness
*For any* detected format export, the bundler should create the correct bundle structure for that format and platform
**Validates: Requirements 7.2, 7.3, 7.4, 7.5, 7.6**

### LV2-Specific Properties

Property 12: LV2 manifest validity
*For any* LV2 plugin, the generated manifest.ttl file should be valid RDF/Turtle syntax and contain all required LV2 metadata
**Validates: Requirements 4.1**

