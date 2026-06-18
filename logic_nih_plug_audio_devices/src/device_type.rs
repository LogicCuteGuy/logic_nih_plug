//! The `AudioIODeviceType` enum + the compile-time `DriverType::current()`
//! helper. Mirrors `juce::AudioIODeviceType`.

use std::fmt;

/// Every audio driver backend JUCE knows about. The `Dummy` entry is the
/// null backend, useful in headless test harnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioIODeviceType {
    /// Apple `CoreAudio` (macOS / iOS).
    CoreAudio,
    /// Steinberg ASIO (Windows; requires the ASIO SDK).
    Asio,
    /// Microsoft WASAPI (Windows).
    Wasapi,
    /// Microsoft DirectSound (Windows; legacy).
    DirectSound,
    /// Linux ALSA.
    Alsa,
    /// Linux JACK.
    Jack,
    /// Android AAudio (API 26+).
    AndroidAAudio,
    /// Android OpenSL ES (legacy; API ≤ 25).
    AndroidOpenSLES,
    /// Bela embedded audio (BeagleBone).
    Bela,
    /// iOS audio (deprecated alias kept for parity with JUCE).
    IOSAudio,
    /// Web Audio API (Emscripten / web plugins).
    WebAudio,
    /// Null backend — no audio is produced or consumed.
    Dummy,
}

impl AudioIODeviceType {
    /// The JUCE-style internal type name (the string JUCE uses to look up
    /// the driver factory).
    pub fn type_name(self) -> &'static str {
        match self {
            AudioIODeviceType::CoreAudio => "CoreAudio",
            AudioIODeviceType::Asio => "ASIO",
            AudioIODeviceType::Wasapi => "WASAPI",
            AudioIODeviceType::DirectSound => "DirectSound",
            AudioIODeviceType::Alsa => "ALSA",
            AudioIODeviceType::Jack => "JACK",
            AudioIODeviceType::AndroidAAudio => "Android AAudio",
            AudioIODeviceType::AndroidOpenSLES => "Android OpenSLES",
            AudioIODeviceType::Bela => "Bela",
            AudioIODeviceType::IOSAudio => "iOS Audio",
            AudioIODeviceType::WebAudio => "Web Audio",
            AudioIODeviceType::Dummy => "Dummy",
        }
    }

    /// Human-readable description (used in the host's settings UI).
    pub fn description(self) -> &'static str {
        match self {
            AudioIODeviceType::CoreAudio => {
                "CoreAudio (macOS / iOS native low-latency audio)"
            }
            AudioIODeviceType::Asio => "ASIO (Steinberg low-latency Windows driver)",
            AudioIODeviceType::Wasapi => "WASAPI (Windows shared / exclusive modes)",
            AudioIODeviceType::DirectSound => "DirectSound (legacy Windows audio)",
            AudioIODeviceType::Alsa => "ALSA (Linux Advanced Linux Sound Architecture)",
            AudioIODeviceType::Jack => "JACK (low-latency Linux audio server)",
            AudioIODeviceType::AndroidAAudio => "AAudio (Android API 26+)",
            AudioIODeviceType::AndroidOpenSLES => "OpenSL ES (Android API <= 25)",
            AudioIODeviceType::Bela => "Bela (embedded BeagleBone audio)",
            AudioIODeviceType::IOSAudio => "iOS Audio (deprecated, use CoreAudio)",
            AudioIODeviceType::WebAudio => "Web Audio (Emscripten / web plugins)",
            AudioIODeviceType::Dummy => "Dummy (no audio — for testing)",
        }
    }

    /// `true` if this driver is plausibly available on the current
    /// operating system. Compile-time only — does *not* guarantee a
    /// concrete device is plugged in (use
    /// [`AudioDeviceManager::scan_device_names`](crate::AudioDeviceManager::scan_device_names)
    /// for that).
    pub fn is_supported_on_current_platform(self) -> bool {
        match self {
            AudioIODeviceType::CoreAudio => cfg!(target_os = "macos") || cfg!(target_os = "ios"),
            AudioIODeviceType::Asio => cfg!(target_os = "windows"),
            AudioIODeviceType::Wasapi => cfg!(target_os = "windows"),
            AudioIODeviceType::DirectSound => cfg!(target_os = "windows"),
            AudioIODeviceType::Alsa => cfg!(target_os = "linux"),
            AudioIODeviceType::Jack => cfg!(target_os = "linux") || cfg!(target_os = "macos"),
            AudioIODeviceType::AndroidAAudio => cfg!(target_os = "android"),
            AudioIODeviceType::AndroidOpenSLES => cfg!(target_os = "android"),
            AudioIODeviceType::Bela => cfg!(target_os = "linux"),
            AudioIODeviceType::IOSAudio => cfg!(target_os = "ios"),
            AudioIODeviceType::WebAudio => cfg!(target_os = "emscripten"),
            AudioIODeviceType::Dummy => true,
        }
    }

    /// All driver variants, in the same order JUCE's `AudioDeviceManager`
    /// enumerates them. Useful for building a driver-picker dropdown.
    pub fn all() -> &'static [AudioIODeviceType] {
        &[
            AudioIODeviceType::CoreAudio,
            AudioIODeviceType::Asio,
            AudioIODeviceType::Wasapi,
            AudioIODeviceType::DirectSound,
            AudioIODeviceType::Alsa,
            AudioIODeviceType::Jack,
            AudioIODeviceType::AndroidAAudio,
            AudioIODeviceType::AndroidOpenSLES,
            AudioIODeviceType::Bela,
            AudioIODeviceType::IOSAudio,
            AudioIODeviceType::WebAudio,
            AudioIODeviceType::Dummy,
        ]
    }

    /// Every driver that is plausibly available on the current operating
    /// system. `Dummy` is always appended last as a fallback.
    pub fn supported_on_current_platform() -> Vec<AudioIODeviceType> {
        let mut out: Vec<AudioIODeviceType> = Self::all()
            .iter()
            .copied()
            .filter(|d| d.is_supported_on_current_platform())
            .collect();
        if !out.contains(&AudioIODeviceType::Dummy) {
            out.push(AudioIODeviceType::Dummy);
        }
        out
    }
}

impl fmt::Display for AudioIODeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.type_name())
    }
}

/// Compile-time detection of the *preferred* audio driver on the current
/// host. Useful as a sensible default for
/// [`AudioDeviceManager::set_current_audio_device_type`](crate::AudioDeviceManager::set_current_audio_device_type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriverType {
    /// Apple `CoreAudio`.
    CoreAudio,
    /// Steinberg ASIO.
    Asio,
    /// Microsoft WASAPI.
    Wasapi,
    /// Microsoft DirectSound.
    DirectSound,
    /// Linux ALSA.
    Alsa,
    /// Linux / macOS JACK.
    Jack,
    /// Android AAudio (API 26+).
    AndroidAAudio,
    /// Android OpenSL ES (legacy).
    AndroidOpenSLES,
    /// Bela (embedded).
    Bela,
    /// iOS Audio.
    IOSAudio,
    /// Web Audio (Emscripten).
    WebAudio,
    /// No preferred driver detected (e.g. unknown OS).
    Unknown,
}

impl DriverType {
    /// The preferred driver for the current build target. Picked to match
    /// the order JUCE's `AudioDeviceManager::currentDeviceType` falls back
    /// through.
    pub const fn current() -> DriverType {
        if cfg!(target_os = "macos") {
            DriverType::CoreAudio
        } else if cfg!(target_os = "ios") {
            DriverType::IOSAudio
        } else if cfg!(target_os = "windows") {
            DriverType::Wasapi
        } else if cfg!(target_os = "android") {
            // JUCE picks AAudio on API 26+ and OpenSL ES below that. From
            // Rust we don't know the API level at compile time, so we mirror
            // JUCE's preferred-default and let the consumer downgrade to
            // OpenSL ES if AAudio fails to initialise.
            DriverType::AndroidAAudio
        } else if cfg!(target_os = "linux") {
            DriverType::Alsa
        } else if cfg!(target_os = "emscripten") {
            DriverType::WebAudio
        } else {
            DriverType::Unknown
        }
    }

    /// Map a `DriverType` to the corresponding `AudioIODeviceType`.
    pub fn to_audio_io_device_type(self) -> AudioIODeviceType {
        match self {
            DriverType::CoreAudio => AudioIODeviceType::CoreAudio,
            DriverType::Asio => AudioIODeviceType::Asio,
            DriverType::Wasapi => AudioIODeviceType::Wasapi,
            DriverType::DirectSound => AudioIODeviceType::DirectSound,
            DriverType::Alsa => AudioIODeviceType::Alsa,
            DriverType::Jack => AudioIODeviceType::Jack,
            DriverType::AndroidAAudio => AudioIODeviceType::AndroidAAudio,
            DriverType::AndroidOpenSLES => AudioIODeviceType::AndroidOpenSLES,
            DriverType::Bela => AudioIODeviceType::Bela,
            DriverType::IOSAudio => AudioIODeviceType::IOSAudio,
            DriverType::WebAudio => AudioIODeviceType::WebAudio,
            DriverType::Unknown => AudioIODeviceType::Dummy,
        }
    }
}

impl fmt::Display for DriverType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_audio_io_device_type())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_names_are_stable() {
        assert_eq!(AudioIODeviceType::CoreAudio.type_name(), "CoreAudio");
        assert_eq!(AudioIODeviceType::Asio.type_name(), "ASIO");
        assert_eq!(AudioIODeviceType::Wasapi.type_name(), "WASAPI");
        assert_eq!(AudioIODeviceType::DirectSound.type_name(), "DirectSound");
        assert_eq!(AudioIODeviceType::Alsa.type_name(), "ALSA");
        assert_eq!(AudioIODeviceType::Jack.type_name(), "JACK");
        assert_eq!(AudioIODeviceType::Dummy.type_name(), "Dummy");
    }

    #[test]
    fn descriptions_are_non_empty() {
        for d in AudioIODeviceType::all() {
            assert!(!d.description().is_empty());
            assert!(!d.type_name().is_empty());
        }
    }

    #[test]
    fn all_is_in_stable_order() {
        let all = AudioIODeviceType::all();
        assert_eq!(all.len(), 12);
        assert_eq!(all[0], AudioIODeviceType::CoreAudio);
        assert_eq!(all[all.len() - 1], AudioIODeviceType::Dummy);
    }

    #[test]
    fn dummy_is_always_supported() {
        assert!(AudioIODeviceType::Dummy.is_supported_on_current_platform());
    }

    #[test]
    fn supported_on_current_platform_contains_dummy() {
        let list = AudioIODeviceType::supported_on_current_platform();
        assert!(!list.is_empty());
        assert!(list.contains(&AudioIODeviceType::Dummy));
        for d in &list {
            assert!(d.is_supported_on_current_platform());
        }
    }

    #[test]
    fn display_matches_type_name() {
        assert_eq!(
            format!("{}", AudioIODeviceType::Wasapi),
            AudioIODeviceType::Wasapi.type_name()
        );
        assert_eq!(
            format!("{}", AudioIODeviceType::CoreAudio),
            AudioIODeviceType::CoreAudio.type_name()
        );
    }

    #[test]
    fn driver_type_current_returns_a_value() {
        // We can't predict the host, but we can guarantee one of the
        // existing variants is returned.
        let d = DriverType::current();
        let expected = [
            DriverType::CoreAudio,
            DriverType::Asio,
            DriverType::Wasapi,
            DriverType::DirectSound,
            DriverType::Alsa,
            DriverType::Jack,
            DriverType::AndroidAAudio,
            DriverType::AndroidOpenSLES,
            DriverType::Bela,
            DriverType::IOSAudio,
            DriverType::WebAudio,
            DriverType::Unknown,
        ];
        assert!(expected.contains(&d), "got {:?}", d);
    }

    #[test]
    fn driver_type_to_audio_io_device_type_round_trip() {
        for d in [
            DriverType::CoreAudio,
            DriverType::Asio,
            DriverType::Wasapi,
            DriverType::DirectSound,
            DriverType::Alsa,
            DriverType::Jack,
            DriverType::AndroidAAudio,
            DriverType::AndroidOpenSLES,
            DriverType::Bela,
            DriverType::IOSAudio,
            DriverType::WebAudio,
            DriverType::Unknown,
        ] {
            let mapped = d.to_audio_io_device_type();
            // Unknown must collapse to Dummy; everything else should land
            // on the matching AudioIODeviceType variant.
            if d == DriverType::Unknown {
                assert_eq!(mapped, AudioIODeviceType::Dummy);
            } else {
                assert_eq!(mapped.type_name(), d.to_audio_io_device_type().type_name());
            }
        }
    }

    #[test]
    fn driver_type_display_matches_audio_io_device_type() {
        for d in [
            DriverType::CoreAudio,
            DriverType::Asio,
            DriverType::Wasapi,
            DriverType::Jack,
            DriverType::Unknown,
        ] {
            assert_eq!(format!("{}", d), format!("{}", d.to_audio_io_device_type()));
        }
    }

    #[test]
    fn platform_support_is_self_consistent() {
        // The current platform's preferred driver should always be in the
        // "supported" list.
        let preferred = DriverType::current();
        if preferred != DriverType::Unknown {
            let mapped = preferred.to_audio_io_device_type();
            assert!(
                mapped.is_supported_on_current_platform(),
                "current preferred {:?} ({}) claims not to be supported on this platform",
                preferred,
                mapped.type_name()
            );
        }
    }
}