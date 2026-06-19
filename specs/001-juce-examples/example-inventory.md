# JUCE Example Inventory Ledger

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Tasks**: [tasks.md](./tasks.md)
**JUCE source**: <https://github.com/juce-framework/JUCE/tree/master/examples> (commit-pinned snapshot, fetched 2026-06-19)
**Status legend**:
- `pending` — identified but not ported yet
- `ported` — Rust crate exists, builds, doc-tests pass, README + bundler row updated
- `skipped(<module>)` — not portable because a JUCE module is not yet ported to Rust (e.g. `skipped(juce_graphics_3D)`)
- `deferred` — out of scope this iteration (with rationale)
- `existing` — already in the workspace before this feature (no work needed)

## Category rollup

| Category | JUCE examples | Existing | Pending | Ported | Skipped | Deferred |
|---|---|---|---|---|---|---|
| Audio | 13 | 0 | 10 | 3 | 0 | 0 |
| DSP | 9 | 0 | 1 | 8 | 0 | 0 |
| GUI | 27 | 0 | 27 | 0 | 0 | 0 |
| Plugins | 14 | 0 | 13 | 1 | 0 | 0 |
| Utilities | 16 | 0 | 11 | 5 | 0 | 0 |
| DemoRunner | 1 | 0 | 0 | 1 | 0 | 0 |
| GUI | 27 | 0 | 27 | 0 | 0 | 0 |
| Plugins | 14 | 0 | 14 | 0 | 0 | 0 |
| Utilities | 16 | 0 | 16 | 0 | 0 | 0 |
| DemoRunner | 1 | 0 | 1 | 0 | 0 | 0 |
| **Total** | **80** | **0** | **80** | **0** | **0** | **0** |

> Counts above enumerate every `*.h` file in the matching `examples/<Category>/` directory
> (excluding `CMakeLists.txt` and `extern/`/`Builds/`/`JuceLibraryCode/`/`Source/` scaffolding).
> A single JUCE "example" may map to multiple Rust crates when the source contains several
> distinct demos (rare; see WebViewPluginDemo below).

## Audio (13)

| juce_path | rust_crate | kind | status | juce_source_link |
|---|---|---|---|---|
| examples/Audio/AudioAppDemo.h | examples/Audio/audio_app_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioAppDemo.h |
| examples/Audio/AudioLatencyDemo.h | examples/Audio/audio_latency_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioLatencyDemo.h |
| examples/Audio/AudioPlaybackDemo.h | examples/Audio/audio_playback_demo | standalone | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioPlaybackDemo.h |
| examples/Audio/AudioRecordingDemo.h | examples/Audio/audio_recording_demo | standalone | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioRecordingDemo.h |
| examples/Audio/AudioSettingsDemo.h | examples/Audio/audio_settings_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioSettingsDemo.h |
| examples/Audio/AudioSynthesiserDemo.h | examples/Audio/audio_synthesiser_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioSynthesiserDemo.h |
| examples/Audio/AudioWorkgroupDemo.h | examples/Audio/audio_workgroup_demo | standalone | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioWorkgroupDemo.h |
| examples/Audio/CapabilityInquiryDemo.h | examples/Audio/capability_inquiry_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/CapabilityInquiryDemo.h |
| examples/Audio/MPEDemo.h | examples/Audio/mpe_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/MPEDemo.h |
| examples/Audio/MidiDemo.h | examples/Audio/midi_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/MidiDemo.h |
| examples/Audio/PluckedStringsDemo.h | examples/Audio/plucked_strings_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/PluckedStringsDemo.h |
| examples/Audio/SimpleFFTDemo.h | examples/Audio/simple_fft_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/SimpleFFTDemo.h |
| examples/Audio/UmpDemo.h | examples/Audio/ump_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Audio/UmpDemo.h |

## DSP (9)

| juce_path | rust_crate | kind | status | juce_source_link |
|---|---|---|---|---|
| examples/DSP/ConvolutionDemo.h | plugins/examples/dsp/juce_convolution_demo | plugin | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/ConvolutionDemo.h |
| examples/DSP/FIRFilterDemo.h | plugins/examples/dsp/juce_fir_filter_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/FIRFilterDemo.h |
| examples/DSP/GainDemo.h | plugins/examples/dsp/juce_gain_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/GainDemo.h |
| examples/DSP/IIRFilterDemo.h | plugins/examples/dsp/juce_iir_filter_demo | plugin | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/IIRFilterDemo.h |
| examples/DSP/OscillatorDemo.h | plugins/examples/dsp/juce_oscillator_demo | plugin | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/OscillatorDemo.h |
| examples/DSP/OverdriveDemo.h | plugins/examples/dsp/juce_distortion_demo | plugin | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/OverdriveDemo.h |
| examples/DSP/SIMDRegisterDemo.h | plugins/examples/dsp/juce_simd_register_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/SIMDRegisterDemo.h |
| examples/DSP/StateVariableFilterDemo.h | plugins/examples/dsp/juce_state_variable_filter_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/StateVariableFilterDemo.h |
| examples/DSP/WaveShaperTanhDemo.h | plugins/examples/dsp/juce_wave_shaper_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/DSP/WaveShaperTanhDemo.h |

## GUI (27)

| juce_path | rust_crate | kind | status | juce_source_link |
|---|---|---|---|---|
| examples/GUI/AccessibilityDemo.h | plugins/examples/gui/juce_accessibility_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/AccessibilityDemo.h |
| examples/GUI/AnimationAppDemo.h | plugins/examples/gui/juce_animation_app_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/AnimationAppDemo.h |
| examples/GUI/AnimationEasingDemo.h | plugins/examples/gui/juce_animation_easing_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/AnimationEasingDemo.h |
| examples/GUI/AnimatorsDemo.h | plugins/examples/gui/juce_animators_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/AnimatorsDemo.h |
| examples/GUI/BouncingBallWavetableDemo.h | plugins/examples/gui/juce_bouncing_ball_wavetable_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/BouncingBallWavetableDemo.h |
| examples/GUI/CameraDemo.h | plugins/examples/gui/juce_camera_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/CameraDemo.h |
| examples/GUI/CodeEditorDemo.h | plugins/examples/gui/juce_code_editor_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/CodeEditorDemo.h |
| examples/GUI/ComponentDemo.h | plugins/examples/gui/juce_component_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/ComponentDemo.h |
| examples/GUI/ComponentDiagnosticsDemo.h | plugins/examples/gui/juce_component_diagnostics_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/ComponentDiagnosticsDemo.h |
| examples/GUI/ComponentTransformsDemo.h | plugins/examples/gui/juce_component_transforms_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/ComponentTransformsDemo.h |
| examples/GUI/DialogsDemo.h | plugins/examples/gui/juce_dialogs_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/DialogsDemo.h |
| examples/GUI/FlexBoxDemo.h | plugins/examples/gui/juce_flexbox_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/FlexBoxDemo.h |
| examples/GUI/FontFeaturesDemo.h | plugins/examples/gui/juce_font_features_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/FontFeaturesDemo.h |
| examples/GUI/FontsDemo.h | plugins/examples/gui/juce_fonts_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/FontsDemo.h |
| examples/GUI/GraphicsDemo.h | plugins/examples/gui/juce_graphics_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/GraphicsDemo.h |
| examples/GUI/GridDemo.h | plugins/examples/gui/juce_grid_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/GridDemo.h |
| examples/GUI/HelloWorldDemo.h | plugins/examples/gui/juce_hello_world_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/HelloWorldDemo.h |
| examples/GUI/ImagesDemo.h | plugins/examples/gui/juce_images_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/ImagesDemo.h |
| examples/GUI/KeyMappingsDemo.h | plugins/examples/gui/juce_key_mappings_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/KeyMappingsDemo.h |
| examples/GUI/LineSpacingDemo.h | plugins/examples/gui/juce_line_spacing_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/LineSpacingDemo.h |
| examples/GUI/LookAndFeelDemo.h | plugins/examples/gui/juce_look_and_feel_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/LookAndFeelDemo.h |
| examples/GUI/MDIDemo.h | plugins/examples/gui/juce_mdi_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/MDIDemo.h |
| examples/GUI/MenusDemo.h | plugins/examples/gui/juce_menus_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/MenusDemo.h |
| examples/GUI/MultiTouchDemo.h | plugins/examples/gui/juce_multi_touch_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/MultiTouchDemo.h |
| examples/GUI/OpenGLAppDemo.h | plugins/examples/gui/juce_opengl_app_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/OpenGLAppDemo.h |
| examples/GUI/OpenGLDemo.h | plugins/examples/gui/juce_opengl_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/OpenGLDemo.h |
| examples/GUI/OpenGLDemo2D.h | plugins/examples/gui/juce_opengl_demo_2d | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/OpenGLDemo2D.h |
| examples/GUI/PropertiesDemo.h | plugins/examples/gui/juce_properties_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/PropertiesDemo.h |
| examples/GUI/VideoDemo.h | plugins/examples/gui/juce_video_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/VideoDemo.h |
| examples/GUI/WebBrowserDemo.h | plugins/examples/gui/juce_web_browser_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/WebBrowserDemo.h |
| examples/GUI/WidgetsDemo.h | plugins/examples/gui/juce_widgets_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/WidgetsDemo.h |
| examples/GUI/WindowsDemo.h | plugins/examples/gui/juce_windows_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/GUI/WindowsDemo.h |

## Plugins (14)

| juce_path | rust_crate | kind | status | juce_source_link |
|---|---|---|---|---|
| examples/Plugins/ARAPluginDemo.h | plugins/examples/plugins/juce_ara_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/ARAPluginDemo.h |
| examples/Plugins/AUv3SynthPluginDemo.h | plugins/examples/plugins/juce_auv3_synth_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/AUv3SynthPluginDemo.h |
| examples/Plugins/ArpeggiatorPluginDemo.h | plugins/examples/plugins/juce_arpeggiator_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/ArpeggiatorPluginDemo.h |
| examples/Plugins/AudioPluginDemo.h | plugins/examples/plugins/juce_audio_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/AudioPluginDemo.h |
| examples/Plugins/DSPModulePluginDemo.h | plugins/examples/plugins/juce_dsp_module_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/DSPModulePluginDemo.h |
| examples/Plugins/GainPluginDemo.h | plugins/examples/plugins/juce_gain_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/GainPluginDemo.h |
| examples/Plugins/HostPluginDemo.h | examples/Plugins/plugin_host_cli + plugins/examples/plugins/juce_audio_plugin_host_egui | plugin-host | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/HostPluginDemo.h |
| examples/Plugins/MidiLoggerPluginDemo.h | plugins/examples/plugins/juce_midi_logger_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/MidiLoggerPluginDemo.h |
| examples/Plugins/MultiOutSynthPluginDemo.h | plugins/examples/plugins/juce_multi_out_synth_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/MultiOutSynthPluginDemo.h |
| examples/Plugins/NoiseGatePluginDemo.h | plugins/examples/plugins/juce_noise_gate_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/NoiseGatePluginDemo.h |
| examples/Plugins/ReaperEmbeddedViewPluginDemo.h | plugins/examples/plugins/juce_reaper_embedded_view_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/ReaperEmbeddedViewPluginDemo.h |
| examples/Plugins/SamplerPluginDemo.h | plugins/examples/plugins/juce_sampler_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/SamplerPluginDemo.h |
| examples/Plugins/SurroundPluginDemo.h | plugins/examples/plugins/juce_surround_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/SurroundPluginDemo.h |
| examples/Plugins/WebViewPluginDemo.h | plugins/examples/plugins/juce_web_view_plugin_demo | plugin | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/WebViewPluginDemo.h |
| examples/Plugins/WebViewPluginDemoGUI/ | plugins/examples/plugins/juce_web_view_plugin_demo_gui | plugin | pending | https://github.com/juce-framework/JUCE/tree/master/examples/Plugins/WebViewPluginDemoGUI |

## Utilities (16)

| juce_path | rust_crate | kind | status | juce_source_link |
|---|---|---|---|---|
| examples/Utilities/AnalyticsCollectionDemo.h | examples/Utilities/analytics_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/AnalyticsCollectionDemo.h |
| examples/Utilities/Box2DDemo.h | examples/Utilities/box2d_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/Box2DDemo.h |
| examples/Utilities/ChildProcessDemo.h | examples/Utilities/child_process_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/ChildProcessDemo.h |
| examples/Utilities/CryptographyDemo.h | examples/Utilities/cryptography_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/CryptographyDemo.h |
| examples/Utilities/InAppPurchasesDemo.h | examples/Utilities/in_app_purchases_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/InAppPurchasesDemo.h |
| examples/Utilities/JavaScriptDemo.h | examples/Utilities/javascript_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/JavaScriptDemo.h |
| examples/Utilities/LiveConstantDemo.h | examples/Utilities/live_constant_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/LiveConstantDemo.h |
| examples/Utilities/MultithreadingDemo.h | examples/Utilities/multithreading_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/MultithreadingDemo.h |
| examples/Utilities/NetworkingDemo.h | examples/Utilities/networking_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/NetworkingDemo.h |
| examples/Utilities/OSCDemo.h | examples/Utilities/osc_sender_demo + osc_receiver_demo | standalone | **ported** | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/OSCDemo.h |
| — | examples/Utilities/wav_reader | standalone | **ported** | https://github.com/juce-framework/JUCE/tree/master/modules/juce_audio_formats |
| — | examples/Utilities/wav_writer | standalone | **ported** | https://github.com/juce-framework/JUCE/tree/master/modules/juce_audio_formats |
| — | examples/Utilities/midi_file_inspector | standalone | **ported** | https://github.com/juce-framework/JUCE/tree/master/modules/juce_audio_formats |
| examples/Utilities/PushNotificationsDemo.h | examples/Utilities/push_notifications_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/PushNotificationsDemo.h |
| examples/Utilities/SystemInfoDemo.h | examples/Utilities/system_info_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/SystemInfoDemo.h |
| examples/Utilities/TimersAndEventsDemo.h | examples/Utilities/timers_and_events_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/TimersAndEventsDemo.h |
| examples/Utilities/UnitTestsDemo.h | examples/Utilities/unit_tests_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/UnitTestsDemo.h |
| examples/Utilities/ValueTreesDemo.h | examples/Utilities/value_trees_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/ValueTreesDemo.h |
| examples/Utilities/XMLandJSONDemo.h | examples/Utilities/xml_and_json_demo | standalone | pending | https://github.com/juce-framework/JUCE/blob/master/examples/Utilities/XMLandJSONDemo.h |

## DemoRunner (1)

| juce_path | rust_crate | kind | status | juce_source_link |
|---|---|---|---|---|
| examples/DemoRunner/Source/* (all categories) | examples/DemoRunner/juce_demorunner | showcase | **ported** | https://github.com/juce-framework/JUCE/tree/master/examples/DemoRunner/Source |

## Existing examples (pre-feature, kept in place)

The `plugins/examples/` directory already contains examples that pre-date this feature.
Their category is recorded here; their `README.md` front-matter is updated to declare it
(per FR-001 and the example-crate-contract):

| Crate | Category | JUCE equivalent | Status |
|---|---|---|---|
| plugins/examples/gain | DSP/Reference | examples/DSP/GainDemo.h | existing |
| plugins/examples/gain_gui_egui | GUI/Backend | — (no direct JUCE equivalent; demonstrates egui) | existing |
| plugins/examples/gain_gui_iced | GUI/Backend | — | existing |
| plugins/examples/gain_gui_vizia | GUI/Backend | — | existing |
| plugins/examples/gain_multi_format | Plugins/Format | — | existing |
| plugins/examples/gain_vst2 | Plugins/Format | — | existing |
| plugins/examples/gain_au | Plugins/Format | — | existing |
| plugins/examples/gain_auv3 | Plugins/Format | — | existing |
| plugins/examples/gain_lv2 | Plugins/Format | — | existing |
| plugins/examples/gain_aax | Plugins/Format | — | existing |
| plugins/examples/byo_gui_gl | GUI/BYO | — | existing |
| plugins/examples/byo_gui_softbuffer | GUI/BYO | — | existing |
| plugins/examples/byo_gui_wgpu | GUI/BYO | — | existing |
| plugins/examples/flexbox_demo | GUI/Layout | examples/GUI/FlexBoxDemo.h | existing |
| plugins/examples/juce_dsp_filter | DSP/Reference | examples/DSP/IIRFilterDemo.h | existing |
| plugins/examples/juce_gui_demo | GUI/Backend | — | existing |
| plugins/examples/juce_multi_module | Plugins/Composite | — | existing |
| plugins/examples/midi_inverter | DSP/MIDI | — | existing |
| plugins/examples/poly_mod_synth | DSP/Synth | — | existing |
| plugins/examples/sine | DSP/Oscillator | — | existing |
| plugins/examples/stft | DSP/Spectral | examples/Audio/SimpleFFTDemo.h | existing |
| plugins/examples/sysex | DSP/MIDI | — | existing |
| plugins/examples/state_variable_filter | DSP/Filter | examples/DSP/StateVariableFilterDemo.h | existing |
| plugins/examples/overdrive | DSP/Distortion | examples/DSP/OverdriveDemo.h | existing |
| plugins/examples/spectrum_analyzer | DSP/Analysis | — | existing |
| plugins/examples/delay | DSP/Time | — | existing |
| plugins/examples/reverb | DSP/Reverb | — | existing |
| plugins/examples/chorus | DSP/Modulation | — | existing |
| plugins/examples/sidechain_compressor | DSP/Dynamics | — | existing |
| plugins/examples/note_expressions | DSP/MIDI | — | existing |
