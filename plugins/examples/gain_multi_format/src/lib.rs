use nih_plug::prelude::*;
use std::sync::Arc;

/// A simple gain plugin demonstrating multi-format export
/// 
/// This plugin exports to VST2, VST3, AU, AUv3, LV2, and CLAP formats,
/// demonstrating how a single plugin implementation can target multiple
/// plugin APIs simultaneously.
struct GainMultiFormat {
    params: Arc<GainMultiFormatParams>,
}

#[derive(Params)]
struct GainMultiFormatParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for GainMultiFormat {
    fn default() -> Self {
        Self {
            params: Arc::new(GainMultiFormatParams::default()),
        }
    }
}

impl Default for GainMultiFormatParams {
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

impl Plugin for GainMultiFormat {
    const NAME: &'static str = "Gain Multi-Format";
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

// VST2 format-specific trait
impl Vst2Plugin for GainMultiFormat {
    const VST2_UNIQUE_ID: i32 = 0x474D4654; // "GMFT"
    const VST2_CATEGORY: Vst2Category = Vst2Category::Effect;
}

// VST3 format-specific trait
impl Vst3Plugin for GainMultiFormat {
    const VST3_CLASS_ID: [u8; 16] = *b"GainMultiFormatP";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

// AU format-specific trait (macOS only)
#[cfg(all(feature = "au", target_os = "macos"))]
impl AuPlugin for GainMultiFormat {
    const AU_TYPE: [u8; 4] = *b"aufx";
    const AU_SUBTYPE: [u8; 4] = *b"gmft";
    const AU_MANUFACTURER: [u8; 4] = *b"Mois";
}

// AUv3 format-specific trait (macOS/iOS only)
#[cfg(all(feature = "auv3", target_os = "macos"))]
impl Auv3Plugin for GainMultiFormat {
    const AUV3_COMPONENT_TYPE: [u8; 4] = *b"aufx";
    const AUV3_COMPONENT_SUBTYPE: [u8; 4] = *b"gmft";
    const AUV3_COMPONENT_MANUFACTURER: [u8; 4] = *b"Mois";
    const AUV3_TAGS: &'static [&'static str] = &["Effects"];
}

// LV2 format-specific trait
impl Lv2Plugin for GainMultiFormat {
    const LV2_URI: &'static str = "https://github.com/robbert-vdh/nih-plug/examples/gain_multi_format";
    const LV2_CATEGORY: Lv2Category = Lv2Category::UtilityPlugin;
}

// CLAP format-specific trait
impl ClapPlugin for GainMultiFormat {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain-multi-format";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A gain plugin demonstrating multi-format export");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

// Export macros for all formats
nih_export_vst2!(GainMultiFormat);
nih_export_vst3!(GainMultiFormat);
#[cfg(all(feature = "au", target_os = "macos"))]
nih_export_au!(GainMultiFormat);
#[cfg(all(feature = "auv3", target_os = "macos"))]
nih_export_auv3!(GainMultiFormat);
nih_export_lv2!(GainMultiFormat);
nih_export_clap!(GainMultiFormat);
