---
category: Utilities
juce_source: examples/Utilities/MidiFileDemo.h
---

# `midi_file_inspector` — Standard MIDI File CLI

## What this example ports

- **JUCE source file**: `examples/Utilities/MidiFileDemo.h`
- **What to learn**: how to use
  `logic_nih_plug_audio_formats::midi_file::MidiFile::read_from` to
  parse a Standard MIDI File (.mid) and introspect its format,
  tracks, tempo, and time signature.

## How it works

1. Accepts an SMF path as a CLI argument.
2. Parses the file and walks the tracks.
3. Prints the format, PPQN, track count, total events, first tempo
   event, and first time-signature event.

## Running

```bash
cargo run -p midi_file_inspector -- examples/midi-assets/single_note.mid
```

## Running the tests

```bash
cargo test -p midi_file_inspector
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 2    | Bad args / missing path |
| 3    | SMF parse or I/O error |

## References

- [`logic_nih_plug_audio_formats::midi_file`](../../../logic_nih_plug_audio_formats/src/midi_file.rs) — `MidiFile`
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec