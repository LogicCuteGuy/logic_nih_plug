//! Benchmarks for DSP operations.
//!
//! This benchmark suite measures the performance of core DSP algorithms
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nih_plug_dsp::filters::IIRFilter;
use nih_plug_dsp::oscillators::{Oscillator, Waveform};

// Benchmark IIR filter processing at different buffer sizes
fn bench_iir_filter_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("iir_filter_processing");
    
    for buffer_size in [64, 128, 256, 512, 1024, 2048].iter() {
        group.throughput(Throughput::Elements(*buffer_size as u64));
        
        // First-order filter
        group.bench_with_input(
            BenchmarkId::new("first_order", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = IIRFilter::new();
                filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
                let input = vec![0.5; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // Second-order filter
        group.bench_with_input(
            BenchmarkId::new("second_order", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = IIRFilter::new();
                filter.set_coefficients(
                    &[0.25, 0.5, 0.25],
                    &[1.0, -0.5, 0.25]
                ).unwrap();
                let input = vec![0.5; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // Third-order filter
        group.bench_with_input(
            BenchmarkId::new("third_order", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = IIRFilter::new();
                filter.set_coefficients(
                    &[0.125, 0.375, 0.375, 0.125],
                    &[1.0, -0.5, 0.25, -0.125]
                ).unwrap();
                let input = vec![0.5; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark single sample processing
fn bench_iir_filter_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("iir_filter_sample");
    
    group.bench_function("first_order", |b| {
        let mut filter = IIRFilter::new();
        filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
        
        b.iter(|| {
            filter.process_sample(black_box(0.5))
        });
    });
    
    group.bench_function("second_order", |b| {
        let mut filter = IIRFilter::new();
        filter.set_coefficients(
            &[0.25, 0.5, 0.25],
            &[1.0, -0.5, 0.25]
        ).unwrap();
        
        b.iter(|| {
            filter.process_sample(black_box(0.5))
        });
    });
    
    group.finish();
}

// Benchmark oscillator generation at different buffer sizes
fn bench_oscillator_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("oscillator_generation");
    
    for buffer_size in [64, 128, 256, 512, 1024, 2048].iter() {
        group.throughput(Throughput::Elements(*buffer_size as u64));
        
        // Sine wave
        group.bench_with_input(
            BenchmarkId::new("sine", buffer_size),
            buffer_size,
            |b, &size| {
                let mut osc = Oscillator::new(44100.0);
                osc.set_frequency(440.0);
                osc.set_waveform(Waveform::Sine);
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    osc.process(black_box(&mut output));
                });
            },
        );
        
        // Saw wave
        group.bench_with_input(
            BenchmarkId::new("saw", buffer_size),
            buffer_size,
            |b, &size| {
                let mut osc = Oscillator::new(44100.0);
                osc.set_frequency(440.0);
                osc.set_waveform(Waveform::Saw);
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    osc.process(black_box(&mut output));
                });
            },
        );
        
        // Square wave
        group.bench_with_input(
            BenchmarkId::new("square", buffer_size),
            buffer_size,
            |b, &size| {
                let mut osc = Oscillator::new(44100.0);
                osc.set_frequency(440.0);
                osc.set_waveform(Waveform::Square);
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    osc.process(black_box(&mut output));
                });
            },
        );
        
        // Triangle wave
        group.bench_with_input(
            BenchmarkId::new("triangle", buffer_size),
            buffer_size,
            |b, &size| {
                let mut osc = Oscillator::new(44100.0);
                osc.set_frequency(440.0);
                osc.set_waveform(Waveform::Triangle);
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    osc.process(black_box(&mut output));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark per-sample oscillator generation
fn bench_oscillator_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("oscillator_sample");
    
    group.bench_function("sine", |b| {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Sine);
        
        b.iter(|| {
            osc.process_sample()
        });
    });
    
    group.bench_function("saw", |b| {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Saw);
        
        b.iter(|| {
            osc.process_sample()
        });
    });
    
    group.bench_function("square", |b| {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Square);
        
        b.iter(|| {
            osc.process_sample()
        });
    });
    
    group.bench_function("triangle", |b| {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Triangle);
        
        b.iter(|| {
            osc.process_sample()
        });
    });
    
    group.finish();
}

// Benchmark frequency modulation
fn bench_oscillator_frequency_modulation(c: &mut Criterion) {
    c.bench_function("oscillator_fm", |b| {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        let mut output = vec![0.0; 1024];
        
        b.iter(|| {
            for (i, sample) in output.iter_mut().enumerate() {
                // Modulate frequency
                let freq = 440.0 + (i as f32 * 0.1).sin() * 100.0;
                osc.set_frequency(freq);
                *sample = osc.process_sample();
            }
        });
    });
}

criterion_group!(
    benches,
    bench_iir_filter_processing,
    bench_iir_filter_sample,
    bench_oscillator_generation,
    bench_oscillator_sample,
    bench_oscillator_frequency_modulation,
);
criterion_main!(benches);
