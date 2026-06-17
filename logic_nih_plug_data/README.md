# logic_nih_plug_data

`ValueTree`, `UndoManager` and `CachedValue` ported from JUCE for nih-plug.

## Features

This crate provides pure-Rust implementations of JUCE's `juce_data_structures`
module:

### 1. `ValueTree`

A hierarchical, observable, reference-counted tree of named properties and child
nodes — ideal for storing plugin state and presets.

```rust
use logic_nih_plug_data::{Identifier, ValueTree};

let tree = ValueTree::new("Preset");
tree.set_property("name", "Init");
tree.set_property("gain", 0.5_f64);

let child = ValueTree::new("Oscillator");
child.set_property("waveform", "saw");
tree.add_child(child, 0);

assert_eq!(tree.num_children(), 1);
assert_eq!(tree.get_string(&"name".into(), ""), "Init");
```

### 2. `UndoManager`

Transaction-based undo/redo for `ValueTree` mutations.

```rust
use logic_nih_plug_data::{UndoManager, ValueTree};

let tree = ValueTree::new("Root");
let undo = UndoManager::new();

tree.set_property_with("gain", 0.5_f64, &undo);
assert!(undo.can_undo());
undo.undo();
assert!(!tree.has_property(&"gain".into()));
```

### 3. `CachedValue<T>`

Typed binding between a `ValueTree` property and a Rust variable.

```rust
use logic_nih_plug_data::{CachedValue, ValueTree};

let tree = ValueTree::new("Synth");
let gain = CachedValue::<f64>::new(&tree, "gain", 1.0);
assert_eq!(gain.get(), 1.0);

gain.set(0.25);
assert_eq!(tree.get_double(&"gain".into(), 0.0), 0.25);
```

## Feature flags

| Feature   | Default | What it adds                                    |
|-----------|---------|-------------------------------------------------|
| `valuetree` | ✅    | `Identifier`, `Value`, `ValueTree`, `CachedValue` |
| `undo`    | ✅      | `UndoManager`, `UndoableAction` and concrete action types |
| `full`    | —       | Re-exports everything (same as the default set) |

## Threading

`ValueTree` and `UndoManager` are single-threaded by default, matching JUCE's
semantics. The underlying `Arc` allows data to be shared, but mutations should
happen from one thread at a time.
