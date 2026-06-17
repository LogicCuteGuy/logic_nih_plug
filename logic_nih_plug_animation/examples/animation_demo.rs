//! Comprehensive animation system demo.
//!
//! This example demonstrates:
//! - Value interpolation with different easing functions
//! - Animation chaining with sequences
//! - Cancellation support
//!
//! Run with: cargo run --example animation_demo --features full

use logic_nih_plug_animation::Animation;
use logic_nih_plug_animation::chaining::AnimationSequence;
use logic_nih_plug_animation::easing::*;

fn main() {
    println!("=== Animation System Demo ===\n");

    // Demo 1: Value Interpolation with Different Easing Functions
    demo_value_interpolation();

    // Demo 2: Animation Chaining
    demo_animation_chaining();

    // Demo 3: Cancellation Support
    demo_cancellation();

    // Demo 4: Dynamic Target Changes
    demo_dynamic_targets();
}

fn demo_value_interpolation() {
    println!("--- Demo 1: Value Interpolation ---");
    
    let easing_functions = vec![
        ("Linear", linear as fn(f32) -> f32),
        ("Ease In Cubic", ease_in_cubic),
        ("Ease Out Cubic", ease_out_cubic),
        ("Ease In Out Cubic", ease_in_out_cubic),
        ("Ease Out Bounce", ease_out_bounce),
        ("Ease Out Elastic", ease_out_elastic),
    ];

    for (name, easing_fn) in easing_functions {
        let mut anim = Animation::new(0.0, 100.0, 1.0, easing_fn);
        anim.start();

        println!("\n{} easing:", name);
        println!("  Time | Value");
        println!("  -----|------");

        for i in 0..=10 {
            let t = i as f32 * 0.1;
            anim.update(0.1);
            println!("  {:.1}s | {:.2}", t, anim.current_value());
        }
    }
    println!();
}

fn demo_animation_chaining() {
    println!("--- Demo 2: Animation Chaining ---");
    
    let mut sequence = AnimationSequence::new();
    
    // Create a sequence of animations with different easing functions
    sequence.add(Animation::new(0.0, 50.0, 1.0, ease_in_cubic));
    sequence.add(Animation::new(50.0, 100.0, 1.0, ease_out_cubic));
    sequence.add(Animation::new(100.0, 0.0, 1.0, ease_in_out_cubic));
    
    sequence.start();
    
    println!("\nAnimating through 3 chained animations:");
    println!("  Time | Animation | Value | Progress");
    println!("  -----|-----------|-------|----------");
    
    let mut time = 0.0;
    while !sequence.is_complete() {
        println!(
            "  {:.1}s | #{:<9} | {:.2} | {:.2}%",
            time,
            sequence.current_index() + 1,
            sequence.current_value(),
            sequence.progress() * 100.0
        );
        sequence.update(0.3);
        time += 0.3;
    }
    
    println!(
        "  {:.1}s | Complete  | {:.2} | {:.2}%",
        time,
        sequence.current_value(),
        sequence.progress() * 100.0
    );
    println!();
}

fn demo_cancellation() {
    println!("--- Demo 3: Cancellation Support ---");
    
    let mut anim = Animation::new(0.0, 100.0, 2.0, ease_in_out_cubic);
    anim.start();
    
    println!("\nStarting animation from 0 to 100 over 2 seconds:");
    
    for i in 0..5 {
        let t = i as f32 * 0.5;
        anim.update(0.5);
        println!("  {:.1}s: value = {:.2}, state = {:?}", t, anim.current_value(), anim.state());
        
        if i == 2 {
            println!("  --> Cancelling animation!");
            anim.cancel();
        }
    }
    
    println!("\nAnimation was cancelled at {:.2}", anim.current_value());
    println!();
}

fn demo_dynamic_targets() {
    println!("--- Demo 4: Dynamic Target Changes ---");
    
    let mut anim = Animation::new(0.0, 100.0, 1.0, ease_out_cubic);
    anim.start();
    
    println!("\nStarting animation from 0 to 100:");
    
    for i in 0..8 {
        let t = i as f32 * 0.25;
        anim.update(0.25);
        println!("  {:.2}s: value = {:.2}", t, anim.current_value());
        
        if i == 2 {
            println!("  --> Changing target to 50!");
            anim.set_target(50.0);
        } else if i == 5 {
            println!("  --> Changing target to 150!");
            anim.set_target(150.0);
        }
    }
    
    println!("\nFinal value: {:.2}", anim.current_value());
    println!();
}
