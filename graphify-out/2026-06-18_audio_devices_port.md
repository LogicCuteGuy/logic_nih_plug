# JUCE port: `logic_nih_plug_audio_devices`

## What was done (2026-06-18)
- TODO §3.2 (Port `juce_audio_devices`) implemented as new crate
  `logic_nih_plug_audio_devices/`.
- 64 lib tests + 1 doc test passing under `--features full` and
  `--no-default-features --features manager`.
- Clippy clean across both feature combos.
- TODO.md / CHANGELOG.md / AGENTS.md all updated.

## Crate shape
- One feature: `manager` (default). `full = ["manager"]`. All the
  pieces are tightly coupled (manager depends on callback trait, which
  depends on the I/O device trait, which depends on AudioDeviceInfo), so
  sub-features would be busy-work.
- Public surface: `AudioDeviceSetup`, `AudioIODeviceType`, `DriverType`,
  `AudioIODevice`, `AudioDeviceInfo`, `AudioIODeviceCallback`,
  `AudioIODeviceCallbackData`, `NullAudioIODeviceCallback`,
  `AudioDeviceManager`, `AudioDeviceManagerListener`,
  `AudioDeviceManagerState`, `MockAudioIODevice`,
  `AudioDevicesError`/`AudioDevicesResult`.

## Key design choices
- **Trait object, not generic** — `Box<dyn AudioIODevice>` in the
  manager. Lets a host swap driver implementations at runtime without
  re-instantiating the manager.
- **`Box<dyn Trait + 'static>` semantics** — `dyn AudioIODevice` defaults
  to `'static`. `get_current_audio_device_mut` had to spell out
  `&mut (dyn AudioIODevice + 'static)` because `+ '_` resolves to the
  function's reference lifetime and fails to coerce down from `'static`.
- **`remove_change_listener` predicate inversion** — `retain(|l| !eq)`
  not `retain(|l| eq)`. Easy to get backwards.
- **`set_current_audio_device` opens the device** — instead of leaving
  the state machine in a half-configured state, installing a device
  calls `open()` with the current setup so that `play()` works
  immediately.
- **Compile-time driver detection** — `DriverType::current()` is a
  `const fn` that picks via `cfg!(target_os = "...")`. Pairs with
  `AudioIODeviceType::is_supported_on_current_platform()` for runtime
  enumeration.
- **No platform-SDK driver bindings** — concrete drivers (cpal,
  coreaudio-rs, asio-sys) are NOT in this crate. The trait surface is
  the integration point; consumers implement `AudioIODevice` for their
  preferred backend.
- **`MockAudioIODevice` is always-compiled + public** — exposed from
  `lib.rs` so hosts can use it as a real device in test harnesses.

## Patterns to remember
- `Rc<Cell<…>>` is not `Send`. Listeners that capture state for
  inspection across `Send` boundaries need `Arc<AtomicUsize>` (or
  `Arc<Mutex<…>>` if mutation is needed).
- `Vec.retain(|x| !predicate(x))` is the idiom for "remove the
  matching element".
- `assert_eq!(x, true)` → `assert!(x)`. `assert_eq!(x, false)` →
  `assert!(!x)`. `let mut x = Default::default(); x.field = ...;` →
  `let x = T { field: ..., ..Default::default() };`. All flagged by
  clippy.
- The `mpsc` of `Set` vs `Box<dyn Trait>` ownership: `Vec<Box<dyn
  Trait>>` for listener lists; `Box<dyn Trait>` for single-owner slots
  like the active device.

## Next unchecked item (TODO.md)
- §4 — Port `juce_core` essentials: `File`, `String` wrapper,
  `Array<T>`/`OwnedArray<T>`/`ReferenceCountedArray<T>`, `Thread`,
  `ThreadPool`, `WaitableEvent`, `Time`, `RelativeTime`,
  `HighResolutionTimer`. Mostly stdlib re-exports; `Thread`/`WaitableEvent`
  may need crossbeam.
