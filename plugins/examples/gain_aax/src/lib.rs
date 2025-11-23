use nih_plug::prelude::*;
use std::sync::Arc;

/// A simple gain plugin demonstrating AAX export
/// 
/// Note: Building AAX plugins requires the AAX SDK from Avid, which requires
/// a developer account. This example will only compile if the AAX SDK is available.
struct GainAax {
    params: Arc<GainAaxParams>,
}

#[derive(Params)]
struct GainAaxParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for GainAax {
    fn default() -> Self {
        Self {
            params: Arc::new(GainAaxParams::default()),
        }
    }
}

impl Default for GainAaxParams {
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

impl Plugin for GainAax {
    const NAME: &'static str = "Gain AAX Example";
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

impl AaxPlugin for GainAax {
    const AAX_MANUFACTURER_ID: [u8; 4] = *b"Mois";
    const AAX_PRODUCT_ID: i32 = 0x47414158; // "GAAX"
    const AAX_CATEGORY: AaxCategory = AaxCategory::EQ;
    const AAX_TYPE_IDS: &'static [AaxTypeId] = &[AaxTypeId::Native];
}

nih_export_aax!(GainAax);
