---
category: Utilities
juce_source: examples/Utilities/OSCDemo.h
---

# `osc_sender_demo` — OSC Sender CLI

## What this example ports

- **JUCE source file**: `examples/Utilities/OSCDemo.h`
- **What to learn**: how to use
  `logic_nih_plug_osc::sender::OscSender` to fire UDP OSC messages
  to a remote peer.

## How it works

1. Connects to a target host/port (defaults to `127.0.0.1:9000`).
2. Builds `OSCMessage`s with mixed-typed arguments
   (Int32 + Float32).
3. Sends `count` messages (default 3).

## Running

```bash
cargo run -p osc_sender_demo -- 127.0.0.1 9000 5
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 3    | Connect / send error |

## References

- [`logic_nih_plug_osc`](../../../logic_nih_plug_osc/src/lib.rs) — OSC sender + receiver
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec