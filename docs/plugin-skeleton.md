# Plugin Skeleton

Minimal end-to-end `gain`-style plugin. Live reference:
[plugins/examples/gain/src/lib.rs](../plugins/examples/gain/src/lib.rs).

```rust
// filepath: plugins/examples/gain/src/lib.rs
use nih_plug::prelude::*;

struct Gain { params: Arc<GainParams> }

#[derive(Params)]
struct GainParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for Gain {
    fn default() -> Self {
        Self { params: Arc::new(GainParams {
            gain: FloatParam::new("Gain", 0.0,
                FloatRange::Linear { min: -60.0, max: 0.0 })
                .with_smoother(SmoothingStyle::Logarithmic(50.0))
                .with_unit(" dB"),
        }) }
    }
}

impl Plugin for Gain {
    const NAME: &'static str = "Gain";
    const VENDOR: &'static str = "...";
    const URL: &'static str = "...";
    const EMAIL: &'static str = "...";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[ /* first = default */ ];
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> { self.params.clone() }
    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers,
               _ctx: &mut impl ProcessContext<Self>) -> ProcessStatus { ProcessStatus::Normal }
    fn editor(&mut self, _ex: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> { None }
    fn initialize(&mut self, _bus: &BusLayout, _buf: &BufferConfig,
                  _ctx: &mut impl InitContext<Self>) -> bool { true }
    fn reset(&mut self) {}
    fn deactivate(&mut self) {}
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "...";
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect];
}
impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"GainPlug_________";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
```

## What to copy for new plugins

- **Oversampling + smoothing** → [plugins/soft_vacuum](../plugins/soft_vacuum)
- **Parameter callbacks** → [plugins/safety_limiter](../plugins/safety_limiter)
- **GUI integration** → [plugins/examples/gain_gui_egui](../plugins/examples/gain_gui_egui)

## Format-specific exports

Multi-format, VST2, AU, AUv3, LV2, AAX variants live in
[plugins/examples](../plugins/examples) and are documented in
[FORMAT_EXAMPLES.md](../plugins/examples/FORMAT_EXAMPLES.md). Keep VST2 and
AAX in separate plugin crates — they collide at the linker.

## Gotchas

- `VST3_CLASS_ID` is `[u8; 16]` — use a 4-char ASCII prefix
- `CLAP_ID` must be unique across all types in one `nih_export_clap!` (debug-asserted)
- Don't put large arrays on the stack (`Box<[T; N]>` instead, see `soft_vacuum`)
- Add a `bundler.toml` entry so CI picks the plugin up via `cargo xtask known-packages`
