use nih_plug::prelude::*;
use std::sync::Arc;

/// A simple gain plugin demonstrating AUv3 export
struct GainAuv3 {
    params: Arc<GainAuv3Params>,
}

#[derive(Params)]
struct GainAuv3Params {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for GainAuv3 {
    fn default() -> Self {
        Self {
            params: Arc::new(GainAuv3Params::default()),
        }
    }
}

impl Default for GainAuv3Params {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for GainAuv3 {
    const NAME: &'static str = "Gain AUv3 Example";
    const VENDOR: &'static str = "Moist Plugins GmbH";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl Auv3Plugin for GainAuv3 {
    const AUV3_COMPONENT_TYPE: [u8; 4] = *b"aufx";
    const AUV3_COMPONENT_SUBTYPE: [u8; 4] = *b"gain";
    const AUV3_COMPONENT_MANUFACTURER: [u8; 4] = *b"Mois";
    const AUV3_TAGS: &'static [&'static str] = &["Effects"];
}

nih_export_auv3!(GainAuv3);
