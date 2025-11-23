# Requirements Document

## Introduction

This document specifies the requirements for extending NIH-plug's plugin export system to support additional audio plugin formats beyond the currently supported VST3 and CLAP formats. The new formats to be supported are VST2, AU (Audio Units), AUv3 (Audio Units v3), LV2, and AAX (Avid Audio eXtension).

## Glossary

- **NIH-plug**: An API-agnostic audio plugin framework written in Rust
- **Export Macro**: A Rust macro that generates the necessary code to expose a plugin in a specific format (e.g., `nih_export_vst3!()`)
- **Plugin Format**: A standardized API specification for audio plugins (VST, AU, AAX, etc.)
- **VST2**: Steinberg's Virtual Studio Technology version 2 plugin format
- **AU**: Apple's Audio Units plugin format for macOS and iOS
- **AUv3**: Audio Units version 3, the modern iOS/macOS plugin format with app extension support
- **LV2**: An open-source plugin standard primarily used on Linux
- **AAX**: Avid Audio eXtension, the plugin format for Pro Tools
- **Plugin Wrapper**: The implementation layer that translates between NIH-plug's API and a specific plugin format's API
- **Cargo Feature**: Rust's conditional compilation mechanism for optional dependencies
- **Bundle**: The platform-specific package format for distributing plugins (e.g., .vst, .component, .aaxplugin)

## Requirements

### Requirement 1

**User Story:** As a plugin developer, I want to export my NIH-plug plugin as a VST2 plugin, so that I can distribute it to users of older DAWs that don't support VST3.

#### Acceptance Criteria

1. WHEN a developer adds `nih_export_vst2!(PluginName)` to their plugin's lib.rs THEN the system SHALL generate a valid VST2 plugin binary
2. WHEN the VST2 plugin is loaded in a VST2-compatible host THEN the system SHALL correctly expose all plugin parameters defined in the Plugin trait
3. WHEN the VST2 plugin processes audio THEN the system SHALL route audio buffers between the host and the Plugin::process() method
4. WHEN the VST2 plugin receives MIDI events THEN the system SHALL translate them to NIH-plug's event format
5. WHERE the plugin defines a GUI THEN the system SHALL expose the editor window to the VST2 host

### Requirement 2

**User Story:** As a macOS plugin developer, I want to export my NIH-plug plugin as an Audio Units (AU) plugin, so that I can distribute it to Logic Pro and GarageBand users.

#### Acceptance Criteria

1. WHEN a developer adds `nih_export_au!(PluginName)` to their plugin's lib.rs THEN the system SHALL generate a valid AU component bundle
2. WHEN the AU plugin is loaded in an AU-compatible host on macOS THEN the system SHALL correctly register all plugin parameters with the AU parameter system
3. WHEN the AU plugin processes audio THEN the system SHALL handle AU's pull-based audio rendering model
4. WHEN the AU plugin receives MIDI events THEN the system SHALL translate AU MIDI events to NIH-plug's event format
5. WHERE the plugin defines preset data THEN the system SHALL support AU's preset save and load mechanisms

### Requirement 3

**User Story:** As an iOS plugin developer, I want to export my NIH-plug plugin as an AUv3 plugin, so that I can distribute it to iOS music production apps.

#### Acceptance Criteria

1. WHEN a developer adds `nih_export_auv3!(PluginName)` to their plugin's lib.rs THEN the system SHALL generate a valid AUv3 app extension
2. WHEN the AUv3 plugin is loaded in an AUv3-compatible host on iOS or macOS THEN the system SHALL correctly expose all plugin parameters
3. WHEN the AUv3 plugin processes audio THEN the system SHALL handle AUv3's real-time audio rendering requirements
4. WHEN the AUv3 plugin is instantiated THEN the system SHALL support AUv3's app extension lifecycle
5. WHERE the plugin defines a GUI THEN the system SHALL provide the view controller to the AUv3 host

### Requirement 4

**User Story:** As a Linux plugin developer, I want to export my NIH-plug plugin as an LV2 plugin, so that I can distribute it to users of Linux DAWs like Ardour and Qtractor.

#### Acceptance Criteria

1. WHEN a developer adds `nih_export_lv2!(PluginName)` to their plugin's lib.rs THEN the system SHALL generate a valid LV2 bundle with required manifest files
2. WHEN the LV2 plugin is loaded in an LV2-compatible host THEN the system SHALL correctly expose all plugin parameters as LV2 control ports
3. WHEN the LV2 plugin processes audio THEN the system SHALL handle LV2's port-based audio processing model
4. WHEN the LV2 plugin receives MIDI events THEN the system SHALL translate LV2 atom events to NIH-plug's event format
5. WHERE the plugin defines state data THEN the system SHALL support LV2's state extension for preset management

### Requirement 5

**User Story:** As a professional plugin developer, I want to export my NIH-plug plugin as an AAX plugin, so that I can distribute it to Pro Tools users.

#### Acceptance Criteria

1. WHEN a developer adds `nih_export_aax!(PluginName)` to their plugin's lib.rs THEN the system SHALL generate a valid AAX plugin bundle
2. WHEN the AAX plugin is loaded in Pro Tools THEN the system SHALL correctly register all plugin parameters with the AAX parameter system
3. WHEN the AAX plugin processes audio THEN the system SHALL handle AAX's processing callbacks and buffer management
4. WHEN the AAX plugin receives MIDI events THEN the system SHALL translate AAX MIDI events to NIH-plug's event format
5. WHERE the plugin requires AAX-specific metadata THEN the system SHALL provide a trait for developers to specify AAX category, manufacturer ID, and product ID

### Requirement 6

**User Story:** As a plugin developer, I want each plugin format to be an optional cargo feature, so that I can choose which formats to support and avoid unnecessary dependencies.

#### Acceptance Criteria

1. WHEN a developer enables the "vst2" cargo feature THEN the system SHALL make the `nih_export_vst2!()` macro available
2. WHEN a developer enables the "au" cargo feature THEN the system SHALL make the `nih_export_au!()` macro available
3. WHEN a developer enables the "auv3" cargo feature THEN the system SHALL make the `nih_export_auv3!()` macro available
4. WHEN a developer enables the "lv2" cargo feature THEN the system SHALL make the `nih_export_lv2!()` macro available
5. WHEN a developer enables the "aax" cargo feature THEN the system SHALL make the `nih_export_aax!()` macro available
6. WHEN a developer does not enable a format's feature THEN the system SHALL not include that format's dependencies in the build

### Requirement 7

**User Story:** As a plugin developer, I want the bundler tool to automatically detect and package all exported plugin formats, so that I can easily distribute my plugins without manual packaging steps.

#### Acceptance Criteria

1. WHEN a developer runs `cargo xtask bundle` THEN the system SHALL detect all `nih_export_<format>!()` macros in the plugin
2. WHEN the bundler detects a VST2 export THEN the system SHALL create a .vst bundle on macOS or .dll on Windows
3. WHEN the bundler detects an AU export THEN the system SHALL create a .component bundle on macOS
4. WHEN the bundler detects an AUv3 export THEN the system SHALL create an app extension bundle on macOS or iOS
5. WHEN the bundler detects an LV2 export THEN the system SHALL create an LV2 bundle directory with manifest.ttl and plugin binary
6. WHEN the bundler detects an AAX export THEN the system SHALL create a .aaxplugin bundle

### Requirement 8

**User Story:** As a plugin developer, I want format-specific plugin traits to provide additional metadata, so that I can specify format-specific requirements like AU type codes or AAX manufacturer IDs.

#### Acceptance Criteria

1. WHERE a plugin exports VST2 format THEN the system SHALL require the plugin to implement a `Vst2Plugin` trait with unique plugin ID
2. WHERE a plugin exports AU format THEN the system SHALL require the plugin to implement an `AuPlugin` trait with manufacturer code, subtype, and type
3. WHERE a plugin exports AUv3 format THEN the system SHALL require the plugin to implement an `Auv3Plugin` trait with component type and tags
4. WHERE a plugin exports LV2 format THEN the system SHALL require the plugin to implement an `Lv2Plugin` trait with URI and category
5. WHERE a plugin exports AAX format THEN the system SHALL require the plugin to implement an `AaxPlugin` trait with manufacturer ID, product ID, and plugin category

### Requirement 9

**User Story:** As a plugin developer, I want comprehensive documentation for each plugin format, so that I can understand format-specific requirements and limitations.

#### Acceptance Criteria

1. WHEN a developer views the documentation for a format export macro THEN the system SHALL provide examples of implementing the format-specific trait
2. WHEN a developer views the documentation for a format export macro THEN the system SHALL document any platform-specific requirements
3. WHEN a developer views the documentation for a format export macro THEN the system SHALL document any licensing considerations for that format
4. WHEN a developer views the documentation for a format export macro THEN the system SHALL provide information about testing the exported plugin
5. WHEN a developer views the documentation for a format export macro THEN the system SHALL document any known limitations or unsupported features

### Requirement 10

**User Story:** As a plugin developer, I want the plugin wrappers to handle parameter automation correctly for each format, so that my plugins respond properly to host automation.

#### Acceptance Criteria

1. WHEN a VST2 host automates a parameter THEN the system SHALL update the corresponding NIH-plug parameter value
2. WHEN an AU host automates a parameter THEN the system SHALL update the corresponding NIH-plug parameter value
3. WHEN an AUv3 host automates a parameter THEN the system SHALL update the corresponding NIH-plug parameter value
4. WHEN an LV2 host automates a parameter THEN the system SHALL update the corresponding NIH-plug parameter value
5. WHEN an AAX host automates a parameter THEN the system SHALL update the corresponding NIH-plug parameter value
6. WHEN a parameter value changes in the plugin THEN the system SHALL notify the host according to the format's parameter change notification mechanism
