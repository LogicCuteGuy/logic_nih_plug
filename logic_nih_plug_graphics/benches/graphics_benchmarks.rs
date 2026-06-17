//! Benchmarks for graphics operations.
//!
//! This benchmark suite measures the performance of 2D drawing primitives
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use logic_nih_plug_graphics::{Color, Graphics};

// Benchmark pixel setting operations
fn bench_set_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_pixel");
    
    let mut graphics = Graphics::new(800, 600).unwrap();
    graphics.set_color(Color::rgb(255, 128, 64));
    
    group.bench_function("single", |b| {
        b.iter(|| {
            graphics.set_pixel(black_box(400), black_box(300));
        });
    });
    
    group.bench_function("1000_pixels", |b| {
        b.iter(|| {
            for i in 0..1000 {
                graphics.set_pixel(black_box(i % 800), black_box(i / 800));
            }
        });
    });
    
    group.finish();
}

// Benchmark rectangle filling at different sizes
fn bench_fill_rect(c: &mut Criterion) {
    let mut group = c.benchmark_group("fill_rect");
    
    let sizes = vec![
        ("10x10", 10, 10),
        ("50x50", 50, 50),
        ("100x100", 100, 100),
        ("200x200", 200, 200),
        ("400x400", 400, 400),
    ];
    
    for (name, width, height) in sizes {
        let pixel_count = width * height;
        group.throughput(Throughput::Elements(pixel_count as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(width, height),
            |b, &(w, h)| {
                let mut graphics = Graphics::new(800, 600).unwrap();
                graphics.set_color(Color::rgb(255, 0, 0));
                
                b.iter(|| {
                    graphics.fill_rect(black_box(100), black_box(100), black_box(w), black_box(h));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark line drawing at different lengths
fn bench_draw_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_line");
    
    let lines = vec![
        ("short_10px", 0, 0, 10, 0),
        ("medium_50px", 0, 0, 50, 0),
        ("long_100px", 0, 0, 100, 0),
        ("diagonal_100px", 0, 0, 100, 100),
        ("long_500px", 0, 0, 500, 0),
    ];
    
    for (name, x1, y1, x2, y2) in lines {
        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let length = (dx * dx + dy * dy).sqrt() as u64;
        group.throughput(Throughput::Elements(length));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(x1, y1, x2, y2),
            |b, &(x1, y1, x2, y2)| {
                let mut graphics = Graphics::new(800, 600).unwrap();
                graphics.set_color(Color::rgb(0, 255, 0));
                
                b.iter(|| {
                    graphics.draw_line(
                        black_box(x1),
                        black_box(y1),
                        black_box(x2),
                        black_box(y2)
                    );
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark circle drawing at different radii
fn bench_draw_circle(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_circle");
    
    let radii = vec![5, 10, 25, 50, 100, 200];
    
    for radius in radii {
        // Approximate number of pixels in circle perimeter
        let perimeter = (2.0 * std::f32::consts::PI * radius as f32) as u64;
        group.throughput(Throughput::Elements(perimeter));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(radius),
            &radius,
            |b, &r| {
                let mut graphics = Graphics::new(800, 600).unwrap();
                graphics.set_color(Color::rgb(0, 0, 255));
                
                b.iter(|| {
                    graphics.draw_circle(black_box(400), black_box(300), black_box(r));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark clearing the entire canvas
fn bench_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("clear");
    
    let sizes = vec![
        ("640x480", 640, 480),
        ("800x600", 800, 600),
        ("1024x768", 1024, 768),
        ("1920x1080", 1920, 1080),
    ];
    
    for (name, width, height) in sizes {
        let pixel_count = width * height;
        group.throughput(Throughput::Elements(pixel_count as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(width, height),
            |b, &(w, h)| {
                let mut graphics = Graphics::new(w, h).unwrap();
                graphics.set_color(Color::rgb(128, 128, 128));
                
                b.iter(|| {
                    graphics.clear();
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark transformation operations
fn bench_transformations(c: &mut Criterion) {
    let mut group = c.benchmark_group("transformations");
    
    group.bench_function("translate", |b| {
        let mut graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            graphics.translate(black_box(10.0), black_box(20.0));
        });
    });
    
    group.bench_function("rotate", |b| {
        let mut graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            graphics.rotate(black_box(0.1));
        });
    });
    
    group.bench_function("scale", |b| {
        let mut graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            graphics.scale(black_box(1.1), black_box(1.1));
        });
    });
    
    group.bench_function("save_restore", |b| {
        let mut graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            graphics.save_transform();
            graphics.translate(10.0, 20.0);
            graphics.restore_transform();
        });
    });
    
    group.finish();
}

// Benchmark complex drawing scenarios
fn bench_complex_scene(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_scene");
    
    group.bench_function("mixed_primitives", |b| {
        let mut graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            // Draw a complex scene with multiple primitives
            graphics.set_color(Color::rgb(255, 0, 0));
            graphics.fill_rect(10, 10, 100, 100);
            
            graphics.set_color(Color::rgb(0, 255, 0));
            graphics.draw_line(0, 0, 800, 600);
            graphics.draw_line(800, 0, 0, 600);
            
            graphics.set_color(Color::rgb(0, 0, 255));
            for i in 0..10 {
                graphics.draw_circle(100 + i * 60, 300, 20);
            }
            
            graphics.set_color(Color::rgb(255, 255, 0));
            for i in 0..5 {
                graphics.fill_rect(200 + i * 100, 400, 80, 80);
            }
        });
    });
    
    group.bench_function("with_transformations", |b| {
        let mut graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            graphics.save_transform();
            
            for i in 0..10 {
                graphics.translate(50.0, 50.0);
                graphics.rotate(0.1);
                graphics.set_color(Color::rgb((i * 25) as u8, 128, 255));
                graphics.fill_rect(0, 0, 40, 40);
            }
            
            graphics.restore_transform();
        });
    });
    
    group.finish();
}

// Benchmark pixel buffer access
fn bench_buffer_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_access");
    
    group.bench_function("read_all_pixels", |b| {
        let graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            let bytes = graphics.as_bytes();
            black_box(bytes.len());
        });
    });
    
    group.bench_function("get_pixel_1000", |b| {
        let graphics = Graphics::new(800, 600).unwrap();
        
        b.iter(|| {
            for i in 0..1000 {
                let _color = graphics.get_pixel(black_box(i % 800), black_box(i / 800));
            }
        });
    });
    
    group.finish();
}

// Benchmark graphics context creation
fn bench_context_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_creation");
    
    let sizes = vec![
        ("640x480", 640, 480),
        ("800x600", 800, 600),
        ("1920x1080", 1920, 1080),
    ];
    
    for (name, width, height) in sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(width, height),
            |b, &(w, h)| {
                b.iter(|| {
                    let _graphics = Graphics::new(black_box(w), black_box(h)).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_set_pixel,
    bench_fill_rect,
    bench_draw_line,
    bench_draw_circle,
    bench_clear,
    bench_transformations,
    bench_complex_scene,
    bench_buffer_access,
    bench_context_creation,
);
criterion_main!(benches);
