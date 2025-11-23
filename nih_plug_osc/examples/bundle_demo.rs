//! Example demonstrating OSC bundle functionality.
//!
//! This example shows how to:
//! - Create bundles with timestamped message groups
//! - Use the BundleBuilder for fluent API
//! - Work with nested bundles
//! - Use BundleUtils for filtering, flattening, and merging

use nih_plug_osc::{
    bundles::{BundleBuilder, BundleUtils},
    OscBundle, OscMessage, OscTime, OscType,
};

fn main() {
    println!("=== OSC Bundle Demo ===\n");

    // Example 1: Creating a simple bundle
    println!("1. Simple Bundle:");
    let mut simple_bundle = OscBundle::immediate();
    simple_bundle.add_message(OscMessage::new("/synth/note", vec![OscType::Int(60)]));
    simple_bundle.add_message(OscMessage::new(
        "/synth/velocity",
        vec![OscType::Float(0.8)],
    ));
    println!("   Created bundle with {} messages", simple_bundle.packets.len());
    println!("   Time tag: immediate = {}\n", simple_bundle.time_tag.is_immediate());

    // Example 2: Using BundleBuilder
    println!("2. Bundle Builder:");
    let builder_bundle = BundleBuilder::new()
        .add_message(OscMessage::new("/test1", vec![OscType::Int(1)]))
        .add_message(OscMessage::new("/test2", vec![OscType::Int(2)]))
        .add_message(OscMessage::new("/test3", vec![OscType::Int(3)]))
        .build();
    println!("   Built bundle with {} messages\n", builder_bundle.packets.len());

    // Example 3: Scheduled bundle with specific time tag
    println!("3. Scheduled Bundle:");
    let future_time = OscTime::new(3600, 0); // 1 hour from epoch
    let scheduled_bundle = BundleBuilder::new()
        .with_time_tag(future_time)
        .add_message(OscMessage::new("/trigger", vec![]))
        .build();
    println!("   Time tag: seconds={}, fractional={}\n", 
             scheduled_bundle.time_tag.seconds, 
             scheduled_bundle.time_tag.fractional);

    // Example 4: Nested bundles
    println!("4. Nested Bundles:");
    let inner_bundle = BundleBuilder::new()
        .add_message(OscMessage::new("/inner/msg1", vec![OscType::Int(1)]))
        .add_message(OscMessage::new("/inner/msg2", vec![OscType::Int(2)]))
        .build();

    let outer_bundle = BundleBuilder::new()
        .add_bundle(inner_bundle)
        .add_message(OscMessage::new("/outer/msg", vec![OscType::Int(3)]))
        .build();

    println!("   Outer bundle packets: {}", outer_bundle.packets.len());
    println!("   Total messages: {}", BundleUtils::count_messages(&outer_bundle));
    println!("   Nesting depth: {}\n", BundleUtils::depth(&outer_bundle));

    // Example 5: Flattening nested bundles
    println!("5. Flattening:");
    let flattened = BundleUtils::flatten(&outer_bundle);
    println!("   Flattened {} messages:", flattened.len());
    for msg in &flattened {
        println!("     - {}", msg.address);
    }
    println!();

    // Example 6: Filtering by address pattern
    println!("6. Filtering:");
    let mut mixed_bundle = OscBundle::immediate();
    mixed_bundle.add_message(OscMessage::new("/synth/note", vec![OscType::Int(60)]));
    mixed_bundle.add_message(OscMessage::new("/synth/velocity", vec![OscType::Float(0.8)]));
    mixed_bundle.add_message(OscMessage::new("/effect/reverb", vec![OscType::Float(0.5)]));
    mixed_bundle.add_message(OscMessage::new("/effect/delay", vec![OscType::Float(0.3)]));

    let synth_only = BundleUtils::filter_by_address(&mixed_bundle, "/synth/*");
    println!("   Original bundle: {} messages", BundleUtils::count_messages(&mixed_bundle));
    println!("   Filtered (/synth/*): {} messages", BundleUtils::count_messages(&synth_only));
    
    let effect_only = BundleUtils::filter_by_address(&mixed_bundle, "/effect/*");
    println!("   Filtered (/effect/*): {} messages\n", BundleUtils::count_messages(&effect_only));

    // Example 7: Merging bundles
    println!("7. Merging:");
    let mut bundle1 = OscBundle::immediate();
    bundle1.add_message(OscMessage::new("/part1/msg1", vec![]));
    bundle1.add_message(OscMessage::new("/part1/msg2", vec![]));

    let mut bundle2 = OscBundle::immediate();
    bundle2.add_message(OscMessage::new("/part2/msg1", vec![]));

    let mut bundle3 = OscBundle::immediate();
    bundle3.add_message(OscMessage::new("/part3/msg1", vec![]));
    bundle3.add_message(OscMessage::new("/part3/msg2", vec![]));
    bundle3.add_message(OscMessage::new("/part3/msg3", vec![]));

    let merged = BundleUtils::merge(&[bundle1, bundle2, bundle3]);
    println!("   Merged 3 bundles into 1");
    println!("   Total messages: {}\n", BundleUtils::count_messages(&merged));

    // Example 8: Complex nested filtering
    println!("8. Complex Nested Filtering:");
    let mut level2_synth = OscBundle::immediate();
    level2_synth.add_message(OscMessage::new("/synth/osc1", vec![]));
    level2_synth.add_message(OscMessage::new("/synth/osc2", vec![]));

    let mut level2_effect = OscBundle::immediate();
    level2_effect.add_message(OscMessage::new("/effect/reverb", vec![]));

    let mut level1 = OscBundle::immediate();
    level1.add_bundle(level2_synth);
    level1.add_bundle(level2_effect);
    level1.add_message(OscMessage::new("/synth/filter", vec![]));
    level1.add_message(OscMessage::new("/master/volume", vec![]));

    println!("   Original nested bundle:");
    println!("     Total messages: {}", BundleUtils::count_messages(&level1));
    println!("     Depth: {}", BundleUtils::depth(&level1));

    let synth_filtered = BundleUtils::filter_by_address(&level1, "/synth/*");
    println!("   After filtering for /synth/*:");
    println!("     Messages: {}", BundleUtils::count_messages(&synth_filtered));
    
    let messages = BundleUtils::flatten(&synth_filtered);
    for msg in &messages {
        println!("     - {}", msg.address);
    }
    println!();

    // Example 9: Building a complex musical sequence
    println!("9. Musical Sequence Bundle:");
    let sequence = BundleBuilder::new()
        .with_time_tag(OscTime::immediate())
        .add_message(OscMessage::new("/synth/note_on", vec![
            OscType::Int(60),  // Middle C
            OscType::Float(0.8),  // Velocity
        ]))
        .add_message(OscMessage::new("/synth/filter_cutoff", vec![
            OscType::Float(1000.0),
        ]))
        .add_message(OscMessage::new("/effect/reverb_mix", vec![
            OscType::Float(0.3),
        ]))
        .add_message(OscMessage::new("/effect/delay_time", vec![
            OscType::Float(0.25),
        ]))
        .build();

    println!("   Created musical sequence with {} messages", 
             BundleUtils::count_messages(&sequence));
    let all_messages = BundleUtils::flatten(&sequence);
    for msg in &all_messages {
        println!("     - {} ({} args)", msg.address, msg.arguments.len());
    }

    println!("\n=== Demo Complete ===");
}
