# logic_nih_plug_animation

Animation utilities ported from JUCE for the nih-plug framework.

## Features

This crate provides a complete animation system with the following capabilities:

### 1. Value Interpolation

The `Animation` struct provides smooth value interpolation between start and end values over a specified duration:

```rust
use logic_nih_plug_animation::{Animation, easing::ease_in_out_cubic};

let mut anim = Animation::new(0.0, 100.0, 1.0, ease_in_out_cubic);
anim.start();

// In your update loop
anim.update(delta_time);
let current_value = anim.current_value();
```

### 2. Easing Curve Support

Over 30 easing functions are provided for different animation styles:

- **Linear**: `linear`
- **Quadratic**: `ease_in_quad`, `ease_out_quad`, `ease_in_out_quad`
- **Cubic**: `ease_in_cubic`, `ease_out_cubic`, `ease_in_out_cubic`
- **Quartic**: `ease_in_quart`, `ease_out_quart`, `ease_in_out_quart`
- **Quintic**: `ease_in_quint`, `ease_out_quint`, `ease_in_out_quint`
- **Sine**: `ease_in_sine`, `ease_out_sine`, `ease_in_out_sine`
- **Exponential**: `ease_in_expo`, `ease_out_expo`, `ease_in_out_expo`
- **Circular**: `ease_in_circ`, `ease_out_circ`, `ease_in_out_circ`
- **Back**: `ease_in_back`, `ease_out_back`, `ease_in_out_back`
- **Elastic**: `ease_in_elastic`, `ease_out_elastic`, `ease_in_out_elastic`
- **Bounce**: `ease_in_bounce`, `ease_out_bounce`, `ease_in_out_bounce`

```rust
use logic_nih_plug_animation::easing::*;

// Try different easing functions
let anim1 = Animation::new(0.0, 100.0, 1.0, ease_out_bounce);
let anim2 = Animation::new(0.0, 100.0, 1.0, ease_in_out_elastic);
```

### 3. Animation Chaining

The `AnimationSequence` struct allows you to chain multiple animations to play sequentially:

```rust
use logic_nih_plug_animation::chaining::AnimationSequence;

let mut sequence = AnimationSequence::new();
sequence.add(Animation::new(0.0, 50.0, 1.0, ease_in_cubic));
sequence.add(Animation::new(50.0, 100.0, 1.0, ease_out_cubic));
sequence.add(Animation::new(100.0, 0.0, 1.0, ease_in_out_cubic));

sequence.start();

// In your update loop
sequence.update(delta_time);
let current_value = sequence.current_value();
```

### 4. Cancellation Support

Both individual animations and sequences can be cancelled at any time:

```rust
// Cancel a single animation
anim.cancel();
assert_eq!(anim.state(), AnimationState::Cancelled);

// Cancel a sequence
sequence.cancel();
assert_eq!(sequence.state(), AnimationState::Cancelled);
```

## Additional Features

### Dynamic Target Changes

You can change the target value of an animation while it's running:

```rust
let mut anim = Animation::new(0.0, 100.0, 1.0, ease_out_cubic);
anim.start();
anim.update(0.5); // Halfway through

// Change target to a new value
anim.set_target(200.0);
// Animation will smoothly transition from current value to 200.0
```

### State Management

Animations track their state through the `AnimationState` enum:

- `NotStarted`: Animation hasn't been started yet
- `Running`: Animation is currently active
- `Complete`: Animation has finished
- `Cancelled`: Animation was cancelled

```rust
match anim.state() {
    AnimationState::Running => { /* Update UI */ },
    AnimationState::Complete => { /* Trigger next action */ },
    AnimationState::Cancelled => { /* Clean up */ },
    _ => {}
}
```

### Reset and Jump

Animations can be reset or jumped to the end:

```rust
// Reset to initial state
anim.reset();

// Jump to the end immediately
anim.jump_to_end();
```

## Cargo Features

- `easing`: Enables easing functions (enabled by default)
- `chaining`: Enables animation chaining (enabled by default)
- `full`: Enables all features

```toml
[dependencies]
logic_nih_plug_animation = { version = "0.0.0", features = ["full"] }
```

## Examples

Run the comprehensive demo:

```bash
cargo run --example animation_demo --features full
```

This demonstrates:
- Value interpolation with different easing functions
- Animation chaining with sequences
- Cancellation support
- Dynamic target changes

## Performance

All easing functions are marked with `#[inline]` for optimal performance. The animation system has minimal overhead and is suitable for real-time audio plugin UIs.

## Thread Safety

The animation types are `Send` but not `Sync`, meaning each thread should have its own animation instances. This is appropriate for UI animations which typically run on a single thread.

## License

ISC License - See LICENSE file for details.
