# logic_nih_plug_audio_basics

Audio buffer / channel-set / MIDI message primitives ported from
[JUCE's `juce_audio_basics` module](https://docs.juce.com/master/juce_audio_basics_README.html)
for the `logic_nih_plug` ecosystem.

This crate mirrors the split that JUCE itself uses: an *audio* half
(`AudioSampleBuffer`, `AudioChannelSet`) and a *MIDI* half
(`MidiMessage`, `MidiRPN`, `MidiClock`, `MTC`). Each half is gated behind its
own feature flag so that consumers who only need, say, MIDI message parsing
don't have to pull in the audio-buffer machinery (and vice versa).

## What's included

| Module                                | Purpose                                                                       |
|---------------------------------------|-------------------------------------------------------------------------------|
| [`audio_sample_buffer`](src/audio_sample_buffer.rs) | Non-interleaved (JUCE-default) audio sample container with interleaved converters |
| [`audio_channel_set`](src/audio_channel_set.rs) | Speaker/channel layouts (mono, stereo, 5.1, 7.1, ambisonics, custom)       |
| [`midi_message`](src/midi_message.rs) | MIDI message parser + builder (Note On/Off, CC, pitch bend, sysex, …)         |
| [`midi_rpn`](src/midi_rpn.rs)         | Registered Parameter Number / Non-Registered PN helpers                       |
| [`midi_clock`](src/midi_clock.rs)     | MIDI clock (24 ppqn) sample/tick math                                        |
| [`mtc`](src/mtc.rs)                   | MIDI Time Code quarter-frame helpers                                         |
| [`error`](src/error.rs)               | The unified error enum (`AudioBasicsError`)                                  |

## Feature flags

| Flag     | Default | What it adds                                                                 |
|----------|---------|------------------------------------------------------------------------------|
| `buffer` | ✅      | `AudioSampleBuffer`, `AudioChannelSet`                                       |
| `midi`   | ✅      | `MidiMessage`, `MidiRPN`, `MidiClock`, `MTC`                                 |
| `full`   | —       | Equivalent to the default set                                                |

```toml
[dependencies]
# MIDI-only — pulls in MidiMessage/MidiRPN/MidiClock/MTC but not AudioSampleBuffer.
logic_nih_plug_audio_basics = { version = "0", default-features = false, features = ["midi"] }
```

## Quick start

```rust
use logic_nih_plug_audio_basics::{
    AudioChannelSet, AudioSampleBuffer, MidiMessage,
};

let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 512);
buf.clear();
buf.apply_gain(0.5);

let msg = MidiMessage::note_on(1, 60, 100);
assert!(msg.is_note_on());
assert_eq!(msg.note_number(), Some(60));

let bytes = msg.to_bytes();
let parsed = MidiMessage::parse(&bytes, 0).unwrap();
assert_eq!(parsed, msg);
```

## Relationship to `logic_nih_plug::buffer::Buffer`

`AudioSampleBuffer` is a self-contained, allocation-owning container that's
useful for cross-crate plumbing (loading samples from a WAV file into an audio
graph, copying between channel layouts, etc.). It is **not** a drop-in
replacement for the realtime-safe [`nih_plug::buffer::Buffer`][logic_nih_plug::buffer::Buffer],
which is a fat pointer over the host's preallocated scratch buffers and uses
`unsafe` lifetime juggling for zero allocations. Use `AudioSampleBuffer` when
you need an owned buffer; use `Buffer` when you're inside a plugin's
`process()` callback.

## JUCE parity notes

- `AudioSampleBuffer` matches JUCE's **non-interleaved** storage layout
  (one `Vec<f32>` per channel) and exposes `interleave` / `deinterleave`
  helpers for the interleaved case. JUCE itself stores non-interleaved by
  default and only does interleaving on demand.
- `AudioChannelSet` mirrors the JUCE class of the same name. The
  `Ambisonic(order)` variant corresponds to `(order+1)²` channels and uses the
  ACN ordering with SN3D normalisation (matching JUCE).
- `MidiMessage` is a value type that owns a `Vec<u8>` plus a sample-offset
  timestamp, matching JUCE's `MidiMessage` API.
- `MidiRPN` / `MidiClock` / `MTC` are pure helpers — they don't allocate.
