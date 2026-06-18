//! Speaker layouts / channel sets.
//!
//! [`AudioChannelSet`] is the JUCE-style enumeration of common speaker
//! arrangements (mono, stereo, LRC, 5.1, 7.1, ambisonics, …) along with a
//! catch-all [`AudioChannelSet::Custom`] variant for arbitrary channel
//! counts. The set knows how many channels it has, the standard channel
//! order (left, right, centre, …), and how to label each channel.

use std::fmt;

/// The maximum ambisonic order this crate knows about. Ambisonic channel
/// counts grow quadratically, so capping the order keeps things sane.
pub const MAX_AMBISONIC_ORDER: u8 = 7;

/// The order of an ambisonic channel set.
///
/// `n` ⇒ `(n + 1)²` channels (0th order = mono, 1st order = 4 channels,
/// 2nd order = 9 channels, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AmbisonicOrder(pub u8);

impl AmbisonicOrder {
    /// Construct an `AmbisonicOrder`, rejecting anything above
    /// [`MAX_AMBISONIC_ORDER`].
    pub fn new(order: u8) -> Option<Self> {
        (order <= MAX_AMBISONIC_ORDER).then_some(Self(order))
    }

    /// The underlying raw order value (0..=7).
    #[inline]
    pub fn get(self) -> u8 {
        self.0
    }

    /// The number of channels for this ambisonic order.
    #[inline]
    pub fn num_channels(self) -> usize {
        // (n + 1)^2, but without the integer-overflow risk of squaring u8.
        let n = self.0 as usize;
        (n + 1) * (n + 1)
    }
}

impl fmt::Display for AmbisonicOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FOA-{} ({} ch)", self.0, self.num_channels())
    }
}

/// The high-level role of a single channel inside an [`AudioChannelSet`].
///
/// The mapping is independent of the actual channel index — e.g. in 5.1,
/// index 2 always corresponds to [`ChannelType::Centre`], but in 7.1.4 the
/// top-front channels are at indices 10 and 11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelType {
    /// The single channel in a mono layout.
    Mono,
    /// Left / Right of a stereo pair.
    Left,
    /// Right / Left of a stereo pair.
    Right,
    /// Front Centre.
    Centre,
    /// Low-Frequency Effects (the ".1" in 5.1).
    Lfe,
    /// Left / Right Surround.
    LeftSurround,
    /// Right / Left Surround.
    RightSurround,
    /// Left / Right Side (between front and surround).
    LeftSide,
    /// Right / Left Side.
    RightSide,
    /// Centre Surround (between the two surrounds).
    CentreSurround,
    /// Top Front Left / Right (the ".2" in 7.1.2 — height layer).
    TopFrontLeft,
    /// Top Front Right / Left (the ".2" in 7.1.2 — height layer).
    TopFrontRight,
    /// Top Rear Left / Right (the ".4" in 7.1.4 — top-back layer).
    TopRearLeft,
    /// Top Rear Right / Left (the ".4" in 7.1.4 — top-back layer).
    TopRearRight,
    /// Top Centre (single height channel above the listener).
    TopCentre,
    /// Top Front Centre (height directly in front).
    TopFrontCentre,
    /// Top Rear Centre (height directly behind).
    TopRearCentre,
    /// Ambisonic ACN channel at index `n`. Ambisonic channels are addressed
    /// by their ACN index rather than by a labelled role.
    Ambisonic(usize),
    /// An unnamed / custom channel (only present inside
    /// [`AudioChannelSet::Custom`]).
    Unknown,
}

impl ChannelType {
    /// A short, human-readable label for the channel (`"L"`, `"R"`,
    /// `"C"`, `"LFE"`, …).
    pub fn abbreviation(self) -> &'static str {
        match self {
            ChannelType::Mono => "M",
            ChannelType::Left => "L",
            ChannelType::Right => "R",
            ChannelType::Centre => "C",
            ChannelType::Lfe => "LFE",
            ChannelType::LeftSurround => "Ls",
            ChannelType::RightSurround => "Rs",
            ChannelType::LeftSide => "Lss",
            ChannelType::RightSide => "Rss",
            ChannelType::CentreSurround => "Cs",
            ChannelType::TopFrontLeft => "Tfl",
            ChannelType::TopFrontRight => "Tfr",
            ChannelType::TopRearLeft => "Trl",
            ChannelType::TopRearRight => "Trr",
            ChannelType::TopCentre => "Tc",
            ChannelType::TopFrontCentre => "Tfc",
            ChannelType::TopRearCentre => "Trc",
            ChannelType::Ambisonic(_) => "ACN",
            ChannelType::Unknown => "?",
        }
    }

    /// A longer, human-readable description of the channel's speaker role.
    pub fn description(self) -> &'static str {
        match self {
            ChannelType::Mono => "Mono",
            ChannelType::Left => "Left",
            ChannelType::Right => "Right",
            ChannelType::Centre => "Centre",
            ChannelType::Lfe => "Low-Frequency Effects",
            ChannelType::LeftSurround => "Left Surround",
            ChannelType::RightSurround => "Right Surround",
            ChannelType::LeftSide => "Left Side",
            ChannelType::RightSide => "Right Side",
            ChannelType::CentreSurround => "Centre Surround",
            ChannelType::TopFrontLeft => "Top Front Left",
            ChannelType::TopFrontRight => "Top Front Right",
            ChannelType::TopRearLeft => "Top Rear Left",
            ChannelType::TopRearRight => "Top Rear Right",
            ChannelType::TopCentre => "Top Centre",
            ChannelType::TopFrontCentre => "Top Front Centre",
            ChannelType::TopRearCentre => "Top Rear Centre",
            ChannelType::Ambisonic(n) => match n {
                0 => "Ambisonic ACN 0 (W)",
                1 => "Ambisonic ACN 1 (Y)",
                2 => "Ambisonic ACN 2 (Z)",
                3 => "Ambisonic ACN 3 (X)",
                _ => "Ambisonic ACN channel",
            },
            ChannelType::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbreviation())
    }
}

/// The physical / virtual position of a single speaker inside a layout.
///
/// Positions are returned for the named layouts so that audio code can map
/// them to a 3D position (e.g. for binaural rendering or for laying out a
/// GUI representation of the channel set).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeakerPosition {
    /// The azimuth in degrees, clockwise from front centre, in `[-180, 180]`.
    pub azimuth_deg: f32,
    /// The elevation in degrees, positive upward, in `[-90, 90]`.
    pub elevation_deg: f32,
    /// The nominal distance from the listener in arbitrary units
    /// (always 1.0 for the layouts in this crate).
    pub distance: f32,
}

impl SpeakerPosition {
    /// Construct a speaker position in degrees.
    pub const fn new(azimuth_deg: f32, elevation_deg: f32, distance: f32) -> Self {
        Self {
            azimuth_deg,
            elevation_deg,
            distance,
        }
    }
}

/// The full description of a single channel inside an [`AudioChannelSet`]:
/// its role and its speaker position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelName {
    /// The role of this channel.
    pub channel_type: ChannelType,
    /// The position of the speaker for this channel.
    pub position: SpeakerPosition,
}

impl ChannelName {
    /// Construct a `ChannelName`.
    pub const fn new(channel_type: ChannelType, position: SpeakerPosition) -> Self {
        Self {
            channel_type,
            position,
        }
    }

    /// A short, human-readable label for the channel.
    pub fn abbreviation(&self) -> &'static str {
        self.channel_type.abbreviation()
    }
}

/// A named channel layout.
///
/// `AudioChannelSet` covers everything from mono to 7.1.4 plus ambisonics,
/// and has a `Custom(usize)` escape hatch for arbitrary channel counts.
/// Named layouts always use the *standard* channel ordering — e.g. 5.1 is
/// `(L, R, C, LFE, Ls, Rs)`, never `(L, R, Ls, Rs, C, LFE)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioChannelSet {
    /// A single channel.
    Mono,
    /// Two channels: left and right.
    Stereo,
    /// Three channels: left, right, centre (no LFE).
    Lrc,
    /// Three channels: left, right, surround.
    Lrs,
    /// Four channels: front L/R, surround L/R (no centre).
    Quadraphonic,
    /// Five channels: front L/R/C, surround L/R.
    FiveDotZero,
    /// Five-point-one: front L/R/C, LFE, surround L/R.
    FiveDotOne,
    /// Six-point-one: 5.1 + centre surround.
    SixDotOne,
    /// Seven channels: front L/R/C, side L/R, surround L/R.
    SevenDotZero,
    /// Seven-point-one: front L/R/C, LFE, side L/R, surround L/R.
    SevenDotOne,
    /// Seven-point-one-point-two: 7.1 + top-front L/R.
    SevenDotOnePointTwo,
    /// Seven-point-one-point-four: 7.1 + top-front L/R + top-rear L/R.
    SevenDotOnePointFour,
    /// First / Higher-Order Ambisonics with the given order. Channel count
    /// is `(order + 1)²` and channels are addressed by ACN index.
    Ambisonic(AmbisonicOrder),
    /// A user-defined layout with the given channel count. The channels
    /// inside a `Custom` layout are reported as [`ChannelType::Unknown`]
    /// with a zero-position.
    Custom(usize),
}

impl AudioChannelSet {
    /// The number of audio channels in this layout.
    pub fn num_channels(&self) -> usize {
        match *self {
            AudioChannelSet::Mono => 1,
            AudioChannelSet::Stereo => 2,
            AudioChannelSet::Lrc | AudioChannelSet::Lrs => 3,
            AudioChannelSet::Quadraphonic => 4,
            AudioChannelSet::FiveDotZero => 5,
            AudioChannelSet::FiveDotOne => 6,
            AudioChannelSet::SixDotOne => 7,
            AudioChannelSet::SevenDotZero => 7,
            AudioChannelSet::SevenDotOne => 8,
            AudioChannelSet::SevenDotOnePointTwo => 10,
            AudioChannelSet::SevenDotOnePointFour => 12,
            AudioChannelSet::Ambisonic(order) => order.num_channels(),
            AudioChannelSet::Custom(n) => n,
        }
    }

    /// A short, JUCE-style name for the layout (`"5.1"`, `"7.1.4"`,
    /// `"Ambisonic order 3"`, …).
    pub fn name(&self) -> String {
        match *self {
            AudioChannelSet::Mono => "Mono".to_owned(),
            AudioChannelSet::Stereo => "Stereo".to_owned(),
            AudioChannelSet::Lrc => "LRC".to_owned(),
            AudioChannelSet::Lrs => "LRS".to_owned(),
            AudioChannelSet::Quadraphonic => "Quadraphonic".to_owned(),
            AudioChannelSet::FiveDotZero => "5.0".to_owned(),
            AudioChannelSet::FiveDotOne => "5.1".to_owned(),
            AudioChannelSet::SixDotOne => "6.1".to_owned(),
            AudioChannelSet::SevenDotZero => "7.0".to_owned(),
            AudioChannelSet::SevenDotOne => "7.1".to_owned(),
            AudioChannelSet::SevenDotOnePointTwo => "7.1.2".to_owned(),
            AudioChannelSet::SevenDotOnePointFour => "7.1.4".to_owned(),
            AudioChannelSet::Ambisonic(order) => format!("Ambisonic order {}", order.get()),
            AudioChannelSet::Custom(n) => format!("Custom ({} ch)", n),
        }
    }

    /// A short, human-readable description for the layout (`"5.1
    /// surround"`, `"Stereo"`, …). Useful for UI strings.
    pub fn description(&self) -> &'static str {
        match *self {
            AudioChannelSet::Mono => "Mono",
            AudioChannelSet::Stereo => "Stereo",
            AudioChannelSet::Lrc => "Left / Right / Centre",
            AudioChannelSet::Lrs => "Left / Right / Surround",
            AudioChannelSet::Quadraphonic => "Quadraphonic",
            AudioChannelSet::FiveDotZero => "5.0 surround (no LFE)",
            AudioChannelSet::FiveDotOne => "5.1 surround",
            AudioChannelSet::SixDotOne => "6.1 surround",
            AudioChannelSet::SevenDotZero => "7.0 surround",
            AudioChannelSet::SevenDotOne => "7.1 surround",
            AudioChannelSet::SevenDotOnePointTwo => "7.1.2 surround (with top-front height)",
            AudioChannelSet::SevenDotOnePointFour => "7.1.4 surround (with top-front and top-rear height)",
            AudioChannelSet::Ambisonic(_) => "Ambisonics",
            AudioChannelSet::Custom(_) => "Custom channel layout",
        }
    }

    /// The description of each channel in this layout, in the layout's
    /// standard channel order.
    ///
    /// For [`AudioChannelSet::Custom`], the returned vector contains
    /// `n` [`ChannelName`] entries all with [`ChannelType::Unknown`] and a
    /// zero-position.
    pub fn channel_names(&self) -> Vec<ChannelName> {
        match *self {
            AudioChannelSet::Mono => vec![ch(ChannelType::Mono, 0.0, 0.0)],
            AudioChannelSet::Stereo => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
            ],
            AudioChannelSet::Lrc => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
            ],
            AudioChannelSet::Lrs => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::LeftSurround, -110.0, 0.0),
            ],
            AudioChannelSet::Quadraphonic => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::LeftSurround, -150.0, 0.0),
                ch(ChannelType::RightSurround, 150.0, 0.0),
            ],
            AudioChannelSet::FiveDotZero => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
                ch(ChannelType::LeftSurround, -110.0, 0.0),
                ch(ChannelType::RightSurround, 110.0, 0.0),
            ],
            AudioChannelSet::FiveDotOne => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
                ch(ChannelType::Lfe, 0.0, 0.0),
                ch(ChannelType::LeftSurround, -110.0, 0.0),
                ch(ChannelType::RightSurround, 110.0, 0.0),
            ],
            AudioChannelSet::SixDotOne => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
                ch(ChannelType::Lfe, 0.0, 0.0),
                ch(ChannelType::LeftSurround, -110.0, 0.0),
                ch(ChannelType::RightSurround, 110.0, 0.0),
                ch(ChannelType::CentreSurround, 180.0, 0.0),
            ],
            AudioChannelSet::SevenDotZero => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
                ch(ChannelType::LeftSide, -90.0, 0.0),
                ch(ChannelType::RightSide, 90.0, 0.0),
                ch(ChannelType::LeftSurround, -150.0, 0.0),
                ch(ChannelType::RightSurround, 150.0, 0.0),
            ],
            AudioChannelSet::SevenDotOne => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
                ch(ChannelType::Lfe, 0.0, 0.0),
                ch(ChannelType::LeftSide, -90.0, 0.0),
                ch(ChannelType::RightSide, 90.0, 0.0),
                ch(ChannelType::LeftSurround, -150.0, 0.0),
                ch(ChannelType::RightSurround, 150.0, 0.0),
            ],
            AudioChannelSet::SevenDotOnePointTwo => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
                ch(ChannelType::Lfe, 0.0, 0.0),
                ch(ChannelType::LeftSide, -90.0, 0.0),
                ch(ChannelType::RightSide, 90.0, 0.0),
                ch(ChannelType::LeftSurround, -150.0, 0.0),
                ch(ChannelType::RightSurround, 150.0, 0.0),
                ch(ChannelType::TopFrontLeft, -30.0, 45.0),
                ch(ChannelType::TopFrontRight, 30.0, 45.0),
            ],
            AudioChannelSet::SevenDotOnePointFour => vec![
                ch(ChannelType::Left, -30.0, 0.0),
                ch(ChannelType::Right, 30.0, 0.0),
                ch(ChannelType::Centre, 0.0, 0.0),
                ch(ChannelType::Lfe, 0.0, 0.0),
                ch(ChannelType::LeftSide, -90.0, 0.0),
                ch(ChannelType::RightSide, 90.0, 0.0),
                ch(ChannelType::LeftSurround, -150.0, 0.0),
                ch(ChannelType::RightSurround, 150.0, 0.0),
                ch(ChannelType::TopFrontLeft, -30.0, 45.0),
                ch(ChannelType::TopFrontRight, 30.0, 45.0),
                ch(ChannelType::TopRearLeft, -150.0, 45.0),
                ch(ChannelType::TopRearRight, 150.0, 45.0),
            ],
            AudioChannelSet::Ambisonic(order) => (0..order.num_channels())
                .map(|n| {
                    ChannelName::new(
                        ChannelType::Ambisonic(n),
                        SpeakerPosition::new(0.0, 0.0, 1.0),
                    )
                })
                .collect(),
            AudioChannelSet::Custom(n) => (0..n)
                .map(|_| {
                    ChannelName::new(
                        ChannelType::Unknown,
                        SpeakerPosition::new(0.0, 0.0, 1.0),
                    )
                })
                .collect(),
        }
    }

    /// Returns `true` if this layout is one of the named surround layouts
    /// (anything more channels than stereo that isn't an ambisonic or
    /// custom layout).
    pub fn is_surround(&self) -> bool {
        matches!(
            *self,
            AudioChannelSet::Lrc
                | AudioChannelSet::Lrs
                | AudioChannelSet::Quadraphonic
                | AudioChannelSet::FiveDotZero
                | AudioChannelSet::FiveDotOne
                | AudioChannelSet::SixDotOne
                | AudioChannelSet::SevenDotZero
                | AudioChannelSet::SevenDotOne
                | AudioChannelSet::SevenDotOnePointTwo
                | AudioChannelSet::SevenDotOnePointFour
        )
    }

    /// Returns `true` if this is a stereo layout.
    pub fn is_stereo(&self) -> bool {
        matches!(*self, AudioChannelSet::Stereo)
    }

    /// Returns `true` if this is a mono layout.
    pub fn is_mono(&self) -> bool {
        matches!(*self, AudioChannelSet::Mono)
    }

    /// Returns `true` if this layout is ambisonic.
    pub fn is_ambisonic(&self) -> bool {
        matches!(*self, AudioChannelSet::Ambisonic(_))
    }
}

impl fmt::Display for AudioChannelSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

/// Tiny constructor helper used by [`AudioChannelSet::channel_names`] above
/// to avoid repeating `ChannelName::new` everywhere.
const fn ch(channel_type: ChannelType, azimuth_deg: f32, elevation_deg: f32) -> ChannelName {
    ChannelName::new(
        channel_type,
        SpeakerPosition::new(azimuth_deg, elevation_deg, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambisonic_channel_counts() {
        assert_eq!(AmbisonicOrder::new(0).unwrap().num_channels(), 1);
        assert_eq!(AmbisonicOrder::new(1).unwrap().num_channels(), 4);
        assert_eq!(AmbisonicOrder::new(2).unwrap().num_channels(), 9);
        assert_eq!(AmbisonicOrder::new(3).unwrap().num_channels(), 16);
        assert!(AmbisonicOrder::new(MAX_AMBISONIC_ORDER + 1).is_none());
    }

    #[test]
    fn channel_counts() {
        assert_eq!(AudioChannelSet::Mono.num_channels(), 1);
        assert_eq!(AudioChannelSet::Stereo.num_channels(), 2);
        assert_eq!(AudioChannelSet::Lrc.num_channels(), 3);
        assert_eq!(AudioChannelSet::FiveDotOne.num_channels(), 6);
        assert_eq!(AudioChannelSet::SevenDotOne.num_channels(), 8);
        assert_eq!(AudioChannelSet::SevenDotOnePointTwo.num_channels(), 10);
        assert_eq!(AudioChannelSet::SevenDotOnePointFour.num_channels(), 12);
        assert_eq!(
            AudioChannelSet::Ambisonic(AmbisonicOrder::new(3).unwrap()).num_channels(),
            16
        );
        assert_eq!(AudioChannelSet::Custom(42).num_channels(), 42);
    }

    #[test]
    fn five_dot_one_order() {
        let names = AudioChannelSet::FiveDotOne.channel_names();
        assert_eq!(names.len(), 6);
        assert_eq!(names[0].channel_type, ChannelType::Left);
        assert_eq!(names[1].channel_type, ChannelType::Right);
        assert_eq!(names[2].channel_type, ChannelType::Centre);
        assert_eq!(names[3].channel_type, ChannelType::Lfe);
        assert_eq!(names[4].channel_type, ChannelType::LeftSurround);
        assert_eq!(names[5].channel_type, ChannelType::RightSurround);
    }

    #[test]
    fn custom_channel_names_are_unknown() {
        let names = AudioChannelSet::Custom(3).channel_names();
        assert_eq!(names.len(), 3);
        for n in &names {
            assert_eq!(n.channel_type, ChannelType::Unknown);
        }
    }

    #[test]
    fn kind_predicates() {
        assert!(AudioChannelSet::Mono.is_mono());
        assert!(!AudioChannelSet::Mono.is_stereo());
        assert!(AudioChannelSet::Stereo.is_stereo());
        assert!(!AudioChannelSet::Stereo.is_mono());
        assert!(AudioChannelSet::FiveDotOne.is_surround());
        assert!(!AudioChannelSet::Stereo.is_surround());
        assert!(AudioChannelSet::Ambisonic(AmbisonicOrder::new(2).unwrap()).is_ambisonic());
        assert!(!AudioChannelSet::FiveDotOne.is_ambisonic());
    }

    #[test]
    fn names_and_descriptions_are_nonempty() {
        let layouts = [
            AudioChannelSet::Mono,
            AudioChannelSet::Stereo,
            AudioChannelSet::Lrc,
            AudioChannelSet::Lrs,
            AudioChannelSet::Quadraphonic,
            AudioChannelSet::FiveDotZero,
            AudioChannelSet::FiveDotOne,
            AudioChannelSet::SixDotOne,
            AudioChannelSet::SevenDotZero,
            AudioChannelSet::SevenDotOne,
            AudioChannelSet::SevenDotOnePointTwo,
            AudioChannelSet::SevenDotOnePointFour,
            AudioChannelSet::Ambisonic(AmbisonicOrder::new(3).unwrap()),
            AudioChannelSet::Custom(4),
        ];
        for layout in layouts {
            assert!(!layout.name().is_empty(), "{:?}.name() is empty", layout);
            assert!(!layout.description().is_empty());
            assert_eq!(layout.channel_names().len(), layout.num_channels());
        }
    }
}
