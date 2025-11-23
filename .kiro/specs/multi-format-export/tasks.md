# Implementation Plan

- [x] 1. Set up project structure and dependencies





  - Add cargo features for each format (vst2, au, auv3, lv2, aax)
  - Add dependencies: vst2-sys, coreaudio-sys, lv2-sys (aax-sdk is proprietary)
  - Create wrapper module directories for each format
  - Create plugin trait files for each format
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

- [x] 2. Implement VST2 support







- [x] 2.1 Create VST2 plugin trait and metadata structures


  - Define `Vst2Plugin` trait with unique ID and category constants
  - Create VST2 category enum
  - _Requirements: 1.1, 8.1_

- [x] 2.2 Implement VST2 wrapper core



  - Create `Vst2Wrapper<P: Plugin>` struct
  - Implement VST2 dispatcher function
  - Implement parameter getters/setters with value conversion
  - Implement audio processing callback
  - _Requirements: 1.2, 1.3_

- [x] 2.3 Implement VST2 MIDI event translation


  - Create MIDI event translation from VST2 to NIH-plug format
  - Handle note on/off, CC, pitch bend, aftertouch
  - _Requirements: 1.4_

- [ ]* 2.4 Write property test for VST2 parameter mapping
  - **Property 1: Parameter exposure completeness (VST2)**
  - **Validates: Requirements 1.2**

- [ ]* 2.5 Write property test for VST2 MIDI translation
  - **Property 5: MIDI event translation correctness (VST2)**
  - **Validates: Requirements 1.4**

- [x] 2.6 Create VST2 export macro


  - Implement `nih_export_vst2!()` macro
  - Generate VST2 entry points (VSTPluginMain, etc.)
  - Handle platform-specific entry points
  - _Requirements: 1.1_

- [ ]* 2.7 Write property test for VST2 binary generation
  - **Property 9: Valid plugin binary generation (VST2)**
  - **Validates: Requirements 1.1**

- [x] 3. Implement AU support





- [x] 3.1 Create AU plugin trait and metadata structures


  - Define `AuPlugin` trait with type/subtype/manufacturer codes
  - Create AU type enum (effect, instrument, etc.)
  - _Requirements: 2.1, 8.2_

- [x] 3.2 Implement AU wrapper core


  - Create `AuWrapper<P: Plugin>` struct
  - Implement AU component dispatch
  - Implement pull-based audio rendering
  - Implement AU parameter system integration
  - _Requirements: 2.2, 2.3_

- [x] 3.3 Implement AU MIDI event translation


  - Create MIDI event translation from AU to NIH-plug format
  - Handle AU MIDI event structures
  - _Requirements: 2.4_

- [x] 3.4 Implement AU preset support


  - Implement AU preset save/load
  - Handle AU property-based state management
  - _Requirements: 2.5_

- [ ]* 3.5 Write property test for AU preset round-trip
  - **Property 6: Preset round-trip consistency (AU)**
  - **Validates: Requirements 2.5**

- [x] 3.6 Create AU export macro


  - Implement `nih_export_au!()` macro
  - Generate AU component entry points
  - Handle Component Manager registration
  - _Requirements: 2.1_

- [ ]* 3.7 Write property test for AU parameter mapping
  - **Property 1: Parameter exposure completeness (AU)**
  - **Validates: Requirements 2.2**

- [x] 4. Implement AUv3 support





- [x] 4.1 Create AUv3 plugin trait and metadata structures


  - Define `Auv3Plugin` trait with component type/subtype/manufacturer
  - Create AUv3 tags enum
  - _Requirements: 3.1, 8.3_

- [x] 4.2 Implement AUv3 wrapper core


  - Create `Auv3Wrapper<P: Plugin>` struct
  - Implement AUv3 audio unit subclass
  - Implement real-time audio rendering
  - Implement AUv3 parameter system
  - _Requirements: 3.2, 3.3_

- [x] 4.3 Create AUv3 export macro

  - Implement `nih_export_auv3!()` macro
  - Generate AUv3 app extension structure
  - Handle iOS/macOS differences
  - _Requirements: 3.1_

- [ ]* 4.4 Write property test for AUv3 parameter mapping
  - **Property 1: Parameter exposure completeness (AUv3)**
  - **Validates: Requirements 3.2**

- [x] 5. Implement LV2 support




- [x] 5.1 Create LV2 plugin trait and metadata structures


  - Define `Lv2Plugin` trait with URI and category
  - Create LV2 category enum
  - _Requirements: 4.1, 8.4_

- [x] 5.2 Implement LV2 wrapper core


  - Create `Lv2Wrapper<P: Plugin>` struct
  - Implement LV2 descriptor
  - Implement port-based audio processing
  - Implement LV2 control port parameter mapping
  - _Requirements: 4.2, 4.3_

- [x] 5.3 Implement LV2 MIDI/Atom event translation


  - Create event translation from LV2 atoms to NIH-plug format
  - Handle LV2 MIDI atom events
  - _Requirements: 4.4_

- [x] 5.4 Implement LV2 state extension


  - Implement LV2 state save/restore
  - Handle LV2 state extension protocol
  - _Requirements: 4.5_

- [ ]* 5.5 Write property test for LV2 state round-trip
  - **Property 6: Preset round-trip consistency (LV2)**
  - **Validates: Requirements 4.5**

- [x] 5.6 Implement LV2 manifest generation


  - Create manifest.ttl generator
  - Create plugin.ttl generator
  - Validate RDF/Turtle syntax
  - _Requirements: 4.1_

- [ ]* 5.7 Write property test for LV2 manifest validity
  - **Property 12: LV2 manifest validity**
  - **Validates: Requirements 4.1**

- [x] 5.8 Create LV2 export macro


  - Implement `nih_export_lv2!()` macro
  - Generate LV2 descriptor entry points
  - Integrate manifest generation
  - _Requirements: 4.1_

- [ ]* 5.9 Write property test for LV2 parameter mapping
  - **Property 1: Parameter exposure completeness (LV2)**
  - **Validates: Requirements 4.2**

- [x] 6. Implement AAX support





- [x] 6.1 Create AAX plugin trait and metadata structures


  - Define `AaxPlugin` trait with manufacturer/product IDs
  - Create AAX category and type ID enums
  - _Requirements: 5.1, 8.5_

- [x] 6.2 Implement AAX wrapper core


  - Create `AaxWrapper<P: Plugin>` struct
  - Implement AAX effect interface
  - Implement AAX parameter system
  - Implement chunk-based audio processing
  - _Requirements: 5.2, 5.3_

- [x] 6.3 Implement AAX MIDI event translation


  - Create MIDI event translation from AAX to NIH-plug format
  - Handle AAX MIDI structures
  - _Requirements: 5.4_

- [x] 6.4 Create AAX export macro


  - Implement `nih_export_aax!()` macro
  - Generate AAX entry points
  - Handle AAX SDK integration
  - _Requirements: 5.1_

- [ ]* 6.5 Write property test for AAX parameter mapping
  - **Property 1: Parameter exposure completeness (AAX)**
  - **Validates: Requirements 5.2**

- [x] 7. Implement cross-format parameter automation




- [x] 7.1 Create unified parameter automation handler


  - Implement parameter change notification system
  - Handle format-specific notification mechanisms
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

- [ ]* 7.2 Write property test for parameter automation
  - **Property 2: Parameter automation updates**
  - **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**

- [ ]* 7.3 Write property test for parameter change notifications
  - **Property 3: Parameter change notifications**
  - **Validates: Requirements 10.6**

- [x] 8. Implement audio buffer routing tests





- [x]* 8.1 Write property test for audio buffer routing


  - **Property 4: Audio buffer routing**
  - **Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.3**

- [x] 9. Extend bundler for new formats




- [x] 9.1 Implement format detection in bundler


  - Parse source files for export macros
  - Detect all format exports in plugin
  - _Requirements: 7.1_

- [ ]* 9.2 Write property test for format detection
  - **Property 10: Format detection accuracy**
  - **Validates: Requirements 7.1**

- [x] 9.3 Implement VST2 bundle generation

  - Create .vst bundle structure for macOS
  - Create .dll for Windows
  - Copy binaries to correct locations
  - _Requirements: 7.2_

- [x] 9.4 Implement AU bundle generation

  - Create .component bundle structure
  - Generate Info.plist
  - Copy binaries to correct locations
  - _Requirements: 7.3_

- [x] 9.5 Implement AUv3 bundle generation

  - Create app extension bundle structure
  - Generate Info.plist for app extension
  - Handle iOS/macOS differences
  - _Requirements: 7.4_

- [x] 9.6 Implement LV2 bundle generation

  - Create LV2 bundle directory structure
  - Copy manifest files
  - Copy plugin binary
  - _Requirements: 7.5_

- [x] 9.7 Implement AAX bundle generation

  - Create .aaxplugin bundle structure
  - Handle code signing requirements
  - _Requirements: 7.6_

- [ ]* 9.8 Write property test for bundle structure correctness
  - **Property 11: Bundle structure correctness**
  - **Validates: Requirements 7.2, 7.3, 7.4, 7.5, 7.6**

- [x] 10. Add comprehensive documentation





  - Document each export macro with examples
  - Document format-specific traits
  - Document platform requirements
  - Document licensing considerations
  - Document testing procedures
  - Document known limitations
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 11. Create example plugins for each format





  - Create example plugin using VST2 export
  - Create example plugin using AU export
  - Create example plugin using AUv3 export
  - Create example plugin using LV2 export
  - Create example plugin using AAX export (if SDK available)
  - Create example plugin using multiple formats
  - _Requirements: All_

- [x] 12. Final checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.
