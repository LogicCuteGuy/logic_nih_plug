//! Benchmarks for GUI operations.
//!
//! This benchmark suite measures the performance of GUI layout algorithms
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nih_plug_gui::layout::flexbox::{
    FlexBox, FlexDirection, FlexWrap, JustifyContent, AlignItems, AlignContent, AlignSelf,
    FlexItem, Margin,
};

// Benchmark FlexBox layout with various item counts
fn bench_flexbox_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("flexbox_layout");
    
    for item_count in [5, 10, 20, 50, 100].iter() {
        group.throughput(Throughput::Elements(*item_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("items", item_count),
            item_count,
            |b, &count| {
                let mut flexbox = FlexBox::new();
                flexbox.direction = FlexDirection::Row;
                flexbox.wrap = FlexWrap::Wrap;
                flexbox.justify_content = JustifyContent::SpaceBetween;
                flexbox.align_items = AlignItems::Center;
                
                for _ in 0..count {
                    flexbox.add_item(FlexItem {
                        order: 0,
                        flex_grow: 1.0,
                        flex_shrink: 1.0,
                        flex_basis: 100.0,
                        align_self: AlignSelf::Auto,
                        width: None,
                        height: None,
                        min_width: Some(50.0),
                        min_height: Some(50.0),
                        max_width: None,
                        max_height: None,
                        margin: Margin {
                            top: 5.0,
                            right: 5.0,
                            bottom: 5.0,
                            left: 5.0,
                        },
                    });
                }
                
                b.iter(|| {
                    flexbox.layout(black_box(800.0), black_box(600.0))
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark FlexBox with different directions
fn bench_flexbox_directions(c: &mut Criterion) {
    let mut group = c.benchmark_group("flexbox_directions");
    
    let item_count = 20;
    group.throughput(Throughput::Elements(item_count as u64));
    
    let directions = vec![
        ("row", FlexDirection::Row),
        ("row_reverse", FlexDirection::RowReverse),
        ("column", FlexDirection::Column),
        ("column_reverse", FlexDirection::ColumnReverse),
    ];
    
    for (name, direction) in directions {
        group.bench_function(name, |b| {
            let mut flexbox = FlexBox::new();
            flexbox.direction = direction;
            flexbox.wrap = FlexWrap::NoWrap;
            flexbox.justify_content = JustifyContent::FlexStart;
            flexbox.align_items = AlignItems::Stretch;
            
            for _ in 0..item_count {
                flexbox.add_item(FlexItem {
                    order: 0,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: 100.0,
                    align_self: AlignSelf::Auto,
                    width: None,
                    height: None,
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                    margin: Margin {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    },
                });
            }
            
            b.iter(|| {
                flexbox.layout(black_box(800.0), black_box(600.0))
            });
        });
    }
    
    group.finish();
}

// Benchmark FlexBox with wrapping
fn bench_flexbox_wrapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("flexbox_wrapping");
    
    let item_count = 30;
    group.throughput(Throughput::Elements(item_count as u64));
    
    let wrap_modes = vec![
        ("nowrap", FlexWrap::NoWrap),
        ("wrap", FlexWrap::Wrap),
        ("wrap_reverse", FlexWrap::WrapReverse),
    ];
    
    for (name, wrap) in wrap_modes {
        group.bench_function(name, |b| {
            let mut flexbox = FlexBox::new();
            flexbox.direction = FlexDirection::Row;
            flexbox.wrap = wrap;
            flexbox.justify_content = JustifyContent::FlexStart;
            flexbox.align_items = AlignItems::FlexStart;
            
            for _ in 0..item_count {
                flexbox.add_item(FlexItem {
                    order: 0,
                    flex_grow: 0.0,
                    flex_shrink: 1.0,
                    flex_basis: 100.0,
                    align_self: AlignSelf::Auto,
                    width: Some(100.0),
                    height: Some(100.0),
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                    margin: Margin {
                        top: 5.0,
                        right: 5.0,
                        bottom: 5.0,
                        left: 5.0,
                    },
                });
            }
            
            b.iter(|| {
                flexbox.layout(black_box(800.0), black_box(600.0))
            });
        });
    }
    
    group.finish();
}

// Benchmark FlexBox with different justify-content modes
fn bench_flexbox_justify_content(c: &mut Criterion) {
    let mut group = c.benchmark_group("flexbox_justify_content");
    
    let item_count = 10;
    group.throughput(Throughput::Elements(item_count as u64));
    
    let justify_modes = vec![
        ("flex_start", JustifyContent::FlexStart),
        ("flex_end", JustifyContent::FlexEnd),
        ("center", JustifyContent::Center),
        ("space_between", JustifyContent::SpaceBetween),
        ("space_around", JustifyContent::SpaceAround),
    ];
    
    for (name, justify) in justify_modes {
        group.bench_function(name, |b| {
            let mut flexbox = FlexBox::new();
            flexbox.direction = FlexDirection::Row;
            flexbox.wrap = FlexWrap::NoWrap;
            flexbox.justify_content = justify;
            flexbox.align_items = AlignItems::Center;
            
            for _ in 0..item_count {
                flexbox.add_item(FlexItem {
                    order: 0,
                    flex_grow: 0.0,
                    flex_shrink: 0.0,
                    flex_basis: 50.0,
                    align_self: AlignSelf::Auto,
                    width: Some(50.0),
                    height: Some(50.0),
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                    margin: Margin {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    },
                });
            }
            
            b.iter(|| {
                flexbox.layout(black_box(800.0), black_box(600.0))
            });
        });
    }
    
    group.finish();
}

// Benchmark FlexBox with flex-grow and flex-shrink
fn bench_flexbox_flexible_items(c: &mut Criterion) {
    let mut group = c.benchmark_group("flexbox_flexible_items");
    
    let item_count = 20;
    group.throughput(Throughput::Elements(item_count as u64));
    
    // All items with flex-grow
    group.bench_function("all_grow", |b| {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        flexbox.wrap = FlexWrap::NoWrap;
        flexbox.justify_content = JustifyContent::FlexStart;
        flexbox.align_items = AlignItems::Stretch;
        
        for _ in 0..item_count {
            flexbox.add_item(FlexItem {
                order: 0,
                flex_grow: 1.0,
                flex_shrink: 0.0,
                flex_basis: 0.0,
                align_self: AlignSelf::Auto,
                width: None,
                height: None,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
                margin: Margin {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                },
            });
        }
        
        b.iter(|| {
            flexbox.layout(black_box(800.0), black_box(600.0))
        });
    });
    
    // All items with flex-shrink
    group.bench_function("all_shrink", |b| {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        flexbox.wrap = FlexWrap::NoWrap;
        flexbox.justify_content = JustifyContent::FlexStart;
        flexbox.align_items = AlignItems::Stretch;
        
        for _ in 0..item_count {
            flexbox.add_item(FlexItem {
                order: 0,
                flex_grow: 0.0,
                flex_shrink: 1.0,
                flex_basis: 100.0,
                align_self: AlignSelf::Auto,
                width: None,
                height: None,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
                margin: Margin {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                },
            });
        }
        
        b.iter(|| {
            flexbox.layout(black_box(800.0), black_box(600.0))
        });
    });
    
    // Mixed flex-grow values
    group.bench_function("mixed_grow", |b| {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        flexbox.wrap = FlexWrap::NoWrap;
        flexbox.justify_content = JustifyContent::FlexStart;
        flexbox.align_items = AlignItems::Stretch;
        
        for i in 0..item_count {
            flexbox.add_item(FlexItem {
                order: 0,
                flex_grow: (i % 3 + 1) as f32,
                flex_shrink: 0.0,
                flex_basis: 0.0,
                align_self: AlignSelf::Auto,
                width: None,
                height: None,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
                margin: Margin {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                },
            });
        }
        
        b.iter(|| {
            flexbox.layout(black_box(800.0), black_box(600.0))
        });
    });
    
    group.finish();
}

// Benchmark FlexBox with complex nested layouts
fn bench_flexbox_complex_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("flexbox_complex_layout");
    
    group.bench_function("complex", |b| {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Column;
        flexbox.wrap = FlexWrap::Wrap;
        flexbox.justify_content = JustifyContent::SpaceBetween;
        flexbox.align_items = AlignItems::Stretch;
        flexbox.align_content = AlignContent::SpaceAround;
        
        // Add items with varying properties
        for i in 0..50 {
            flexbox.add_item(FlexItem {
                order: (i % 5) as i32,
                flex_grow: if i % 2 == 0 { 1.0 } else { 0.0 },
                flex_shrink: if i % 3 == 0 { 0.0 } else { 1.0 },
                flex_basis: (50.0 + (i % 10) as f32 * 10.0),
                align_self: match i % 4 {
                    0 => AlignSelf::Auto,
                    1 => AlignSelf::FlexStart,
                    2 => AlignSelf::Center,
                    _ => AlignSelf::FlexEnd,
                },
                width: if i % 5 == 0 { Some(100.0) } else { None },
                height: if i % 7 == 0 { Some(80.0) } else { None },
                min_width: Some(30.0),
                min_height: Some(30.0),
                max_width: if i % 3 == 0 { Some(200.0) } else { None },
                max_height: if i % 4 == 0 { Some(150.0) } else { None },
                margin: Margin {
                    top: (i % 3) as f32 * 2.0,
                    right: (i % 4) as f32 * 2.0,
                    bottom: (i % 3) as f32 * 2.0,
                    left: (i % 4) as f32 * 2.0,
                },
            });
        }
        
        b.iter(|| {
            flexbox.layout(black_box(1200.0), black_box(800.0))
        });
    });
    
    group.finish();
}

// Benchmark FlexBox with different container sizes
fn bench_flexbox_container_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("flexbox_container_sizes");
    
    let item_count = 20;
    
    let sizes = vec![
        ("small", 400.0, 300.0),
        ("medium", 800.0, 600.0),
        ("large", 1600.0, 1200.0),
        ("xlarge", 3200.0, 2400.0),
    ];
    
    for (name, width, height) in sizes {
        group.bench_function(name, |b| {
            let mut flexbox = FlexBox::new();
            flexbox.direction = FlexDirection::Row;
            flexbox.wrap = FlexWrap::Wrap;
            flexbox.justify_content = JustifyContent::SpaceBetween;
            flexbox.align_items = AlignItems::Center;
            
            for _ in 0..item_count {
                flexbox.add_item(FlexItem {
                    order: 0,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: 100.0,
                    align_self: AlignSelf::Auto,
                    width: None,
                    height: None,
                    min_width: Some(50.0),
                    min_height: Some(50.0),
                    max_width: None,
                    max_height: None,
                    margin: Margin {
                        top: 5.0,
                        right: 5.0,
                        bottom: 5.0,
                        left: 5.0,
                    },
                });
            }
            
            b.iter(|| {
                flexbox.layout(black_box(width), black_box(height))
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_flexbox_layout,
    bench_flexbox_directions,
    bench_flexbox_wrapping,
    bench_flexbox_justify_content,
    bench_flexbox_flexible_items,
    bench_flexbox_complex_layout,
    bench_flexbox_container_sizes,
);

criterion_main!(benches);
