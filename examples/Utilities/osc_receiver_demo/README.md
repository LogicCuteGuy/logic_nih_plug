---
category: Utilities
juce_source: examples/Utilities/OSCDemo.h
---

# `osc_receiver_demo` — OSC Receiver CLI

## What this example ports

- **JUCE source file**: `examples/Utilities/OSCDemo.h`
- **What to learn**: how to bind a UDP OSC receiver on a given port
  and register a [`MessageListener`] to capture incoming messages
  into a shared buffer for inspection.

## How it works

1. Binds a UDP socket on the given port (default `9000`).
2. Installs a `CaptureListener` that pushes every received
   `OSCMessage` into a shared `Vec<ReceivedOscMessage>`.
3. Sleeps briefly, then dumps the captured messages to stdout.

## Running

```bash
# terminal A
cargo run -p osc_receiver_demo -- 9000

# terminal B
cargo run -p osc_sender_demo -- 127.0.0.1 9000 3
```

## Running the tests

```bash
cargo test -p osc_receiver_demo
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 3    | Bind or listener error |

## References

- [`logic_nih_plug_osc`](../../../logic_nih_plug_osc/src/lib.rs) — OSC receiver
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec