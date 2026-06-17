//! # FlexBox Layout Demo
//!
//! This example demonstrates the FlexBox layout system from logic_nih_plug_gui
//! with interactive controls for all FlexBox properties and visual feedback.

use crossbeam::atomic::AtomicCell;
use logic_nih_plug::prelude::*;
use logic_nih_plug_gui::layout::flexbox::*;
use std::sync::Arc;

mod editor;

/// FlexBox direction enum for parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum FlexDirectionParam {
    #[name = "Row"]
    Row,
    #[name = "Row Reverse"]
    RowReverse,
    #[name = "Column"]
    Column,
    #[name = "Column Reverse"]
    ColumnReverse,
}

impl From<FlexDirectionParam> for FlexDirection {
    fn from(param: FlexDirectionParam) -> Self {
        match param {
            FlexDirectionParam::Row => FlexDirection::Row,
            FlexDirectionParam::RowReverse => FlexDirection::RowReverse,
            FlexDirectionParam::Column => FlexDirection::Column,
            FlexDirectionParam::ColumnReverse => FlexDirection::ColumnReverse,
        }
    }
}

/// FlexBox wrap enum for parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum FlexWrapParam {
    #[name = "No Wrap"]
    NoWrap,
    #[name = "Wrap"]
    Wrap,
    #[name = "Wrap Reverse"]
    WrapReverse,
}

impl From<FlexWrapParam> for FlexWrap {
    fn from(param: FlexWrapParam) -> Self {
        match param {
            FlexWrapParam::NoWrap => FlexWrap::NoWrap,
            FlexWrapParam::Wrap => FlexWrap::Wrap,
            FlexWrapParam::WrapReverse => FlexWrap::WrapReverse,
        }
    }
}

/// JustifyContent enum for parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum JustifyContentParam {
    #[name = "Flex Start"]
    FlexStart,
    #[name = "Flex End"]
    FlexEnd,
    #[name = "Center"]
    Center,
    #[name = "Space Between"]
    SpaceBetween,
    #[name = "Space Around"]
    SpaceAround,
}

impl From<JustifyContentParam> for JustifyContent {
    fn from(param: JustifyContentParam) -> Self {
        match param {
            JustifyContentParam::FlexStart => JustifyContent::FlexStart,
            JustifyContentParam::FlexEnd => JustifyContent::FlexEnd,
            JustifyContentParam::Center => JustifyContent::Center,
            JustifyContentParam::SpaceBetween => JustifyContent::SpaceBetween,
            JustifyContentParam::SpaceAround => JustifyContent::SpaceAround,
        }
    }
}

/// AlignItems enum for parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum AlignItemsParam {
    #[name = "Flex Start"]
    FlexStart,
    #[name = "Flex End"]
    FlexEnd,
    #[name = "Center"]
    Center,
    #[name = "Stretch"]
    Stretch,
    #[name = "Baseline"]
    Baseline,
}

impl From<AlignItemsParam> for AlignItems {
    fn from(param: AlignItemsParam) -> Self {
        match param {
            AlignItemsParam::FlexStart => AlignItems::FlexStart,
            AlignItemsParam::FlexEnd => AlignItems::FlexEnd,
            AlignItemsParam::Center => AlignItems::Center,
            AlignItemsParam::Stretch => AlignItems::Stretch,
            AlignItemsParam::Baseline => AlignItems::Baseline,
        }
    }
}

/// AlignContent enum for parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum AlignContentParam {
    #[name = "Flex Start"]
    FlexStart,
    #[name = "Flex End"]
    FlexEnd,
    #[name = "Center"]
    Center,
    #[name = "Space Between"]
    SpaceBetween,
    #[name = "Space Around"]
    SpaceAround,
    #[name = "Stretch"]
    Stretch,
}

impl From<AlignContentParam> for AlignContent {
    fn from(param: AlignContentParam) -> Self {
        match param {
            AlignContentParam::FlexStart => AlignContent::FlexStart,
            AlignContentParam::FlexEnd => AlignContent::FlexEnd,
            AlignContentParam::Center => AlignContent::Center,
            AlignContentParam::SpaceBetween => AlignContent::SpaceBetween,
            AlignContentParam::SpaceAround => AlignContent::SpaceAround,
            AlignContentParam::Stretch => AlignContent::Stretch,
        }
    }
}

/// FlexBox Layout Demo plugin
pub struct FlexBoxDemoPlugin {
    params: Arc<FlexBoxParams>,
}

#[derive(Params)]
struct FlexBoxParams {
    /// FlexBox direction
    #[id = "direction"]
    pub direction: EnumParam<FlexDirectionParam>,

    /// FlexBox wrap
    #[id = "wrap"]
    pub wrap: EnumParam<FlexWrapParam>,

    /// Justify content
    #[id = "justify"]
    pub justify_content: EnumParam<JustifyContentParam>,

    /// Align items
    #[id = "align_items"]
    pub align_items: EnumParam<AlignItemsParam>,

    /// Align content
    #[id = "align_content"]
    pub align_content: EnumParam<AlignContentParam>,

    /// Number of items to display
    #[id = "num_items"]
    pub num_items: IntParam,

    /// Container width (for responsive testing)
    #[id = "container_width"]
    pub container_width: FloatParam,

    /// Container height (for responsive testing)
    #[id = "container_height"]
    pub container_height: FloatParam,
}

impl Default for FlexBoxDemoPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(FlexBoxParams::default()),
        }
    }
}

impl Default for FlexBoxParams {
    fn default() -> Self {
        Self {
            direction: EnumParam::new("Direction", FlexDirectionParam::Row),

            wrap: EnumParam::new("Wrap", FlexWrapParam::Wrap),

            justify_content: EnumParam::new("Justify Content", JustifyContentParam::FlexStart),

            align_items: EnumParam::new("Align Items", AlignItemsParam::Stretch),

            align_content: EnumParam::new("Align Content", AlignContentParam::Stretch),

            num_items: IntParam::new(
                "Number of Items",
                5,
                IntRange::Linear { min: 1, max: 20 },
            ),

            container_width: FloatParam::new(
                "Container Width",
                400.0,
                FloatRange::Linear {
                    min: 200.0,
                    max: 800.0,
                },
            )
            .with_unit(" px")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),

            container_height: FloatParam::new(
                "Container Height",
                300.0,
                FloatRange::Linear {
                    min: 150.0,
                    max: 600.0,
                },
            )
            .with_unit(" px")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
        }
    }
}

impl Plugin for FlexBoxDemoPlugin {
    const NAME: &'static str = "FlexBox Layout Demo";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(editor::FlexBoxDemoEditor {
            params: Arc::clone(&self.params),
            scaling_factor: AtomicCell::new(None),
        }))
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Pass audio through unchanged - this is a GUI demo
        for channel_samples in buffer.iter_samples() {
            // Audio passes through
            for _sample in channel_samples {
                // No processing
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for FlexBoxDemoPlugin {
    const CLAP_ID: &'static str = "com.nih-plug.flexbox-demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("FlexBox layout system demonstration with interactive controls");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for FlexBoxDemoPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"FlexBoxDemoXXXXX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(FlexBoxDemoPlugin);
nih_export_vst3!(FlexBoxDemoPlugin);
