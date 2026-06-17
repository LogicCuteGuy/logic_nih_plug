//! Benchmarks for DSP operations.
//!
//! This benchmark suite measures the performance of core DSP algorithms
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use logic_nih_plug_dsp::filters::IIRFilter;
use logic_nih_plug_dsp::oscillators::{Oscillator, Waveform};
use logic_nih_plug_dsp::state_variable::{StateVariableFilter, FilterType};
use logic_nih_plug_dsp::fir::{FIRFilter, FilterDesign, WindowFunction};
use logic_nih_plug_dsp::analysis::fft::FFT;
use logic_nih_plug_dsp::processors::Processor;
use logic_nih_plug_dsp::processors::chain::ProcessorChain;
use logic_nih_plug_dsp::processors::gain::Gain;
use logic_nih_plug_dsp::processors::bias::Bias;
use logic_nih_plug_dsp::processors::waveshaper::{WaveShaper, transfer_functions};

#[cfg(feature = "simd")]
use logic_nih_plug_dsp::simd::optimizations::{SimdStateVariableFilter, SimdFIRFilter};

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

// Benchmark state variable filter performance
fn bench_state_variable_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_variable_filter");
    
    for buffer_size in [64, 128, 256, 512, 1024, 2048].iter() {
        group.throughput(Throughput::Elements(*buffer_size as u64));
        
        // Lowpass filter
        group.bench_with_input(
            BenchmarkId::new("lowpass", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = StateVariableFilter::new();
                filter.prepare(44100.0).unwrap();
                filter.set_type(FilterType::Lowpass);
                filter.set_cutoff(1000.0);
                filter.set_resonance(0.7);
                
                let input = vec![0.5; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // Bandpass filter
        group.bench_with_input(
            BenchmarkId::new("bandpass", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = StateVariableFilter::new();
                filter.prepare(44100.0).unwrap();
                filter.set_type(FilterType::Bandpass);
                filter.set_cutoff(1000.0);
                filter.set_resonance(0.7);
                
                let input = vec![0.5; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // Highpass filter
        group.bench_with_input(
            BenchmarkId::new("highpass", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = StateVariableFilter::new();
                filter.prepare(44100.0).unwrap();
                filter.set_type(FilterType::Highpass);
                filter.set_cutoff(1000.0);
                filter.set_resonance(0.7);
                
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

// Benchmark state variable filter per-sample processing
fn bench_state_variable_filter_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_variable_filter_sample");
    
    group.bench_function("lowpass", |b| {
        let mut filter = StateVariableFilter::new();
        filter.prepare(44100.0).unwrap();
        filter.set_type(FilterType::Lowpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.7);
        
        b.iter(|| {
            filter.process_sample(black_box(0.5))
        });
    });
    
    group.bench_function("bandpass", |b| {
        let mut filter = StateVariableFilter::new();
        filter.prepare(44100.0).unwrap();
        filter.set_type(FilterType::Bandpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.7);
        
        b.iter(|| {
            filter.process_sample(black_box(0.5))
        });
    });
    
    group.bench_function("highpass", |b| {
        let mut filter = StateVariableFilter::new();
        filter.prepare(44100.0).unwrap();
        filter.set_type(FilterType::Highpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.7);
        
        b.iter(|| {
            filter.process_sample(black_box(0.5))
        });
    });
    
    group.finish();
}

// Benchmark FIR filter with various lengths
fn bench_fir_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("fir_filter");
    
    let buffer_size = 1024;
    group.throughput(Throughput::Elements(buffer_size as u64));
    
    // Test with different filter lengths
    for filter_length in [16, 32, 64, 128, 256, 512].iter() {
        let coeffs = FilterDesign::fir_lowpass(
            1000.0,
            44100.0,
            *filter_length,
            WindowFunction::Hamming,
        ).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("length", filter_length),
            &coeffs,
            |b, coeffs| {
                let mut filter = FIRFilter::new(coeffs.clone());
                let input = vec![0.5; buffer_size];
                let mut output = vec![0.0; buffer_size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark FIR filter with different window functions
fn bench_fir_filter_windows(c: &mut Criterion) {
    let mut group = c.benchmark_group("fir_filter_windows");
    
    let buffer_size = 1024;
    let filter_length = 64;
    group.throughput(Throughput::Elements(buffer_size as u64));
    
    let windows = vec![
        ("rectangular", WindowFunction::Rectangular),
        ("triangular", WindowFunction::Triangular),
        ("hann", WindowFunction::Hann),
        ("hamming", WindowFunction::Hamming),
        ("blackman", WindowFunction::Blackman),
        ("blackman_harris", WindowFunction::BlackmanHarris),
    ];
    
    for (name, window) in windows {
        let coeffs = FilterDesign::fir_lowpass(
            1000.0,
            44100.0,
            filter_length,
            window,
        ).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("window", name),
            &coeffs,
            |b, coeffs| {
                let mut filter = FIRFilter::new(coeffs.clone());
                let input = vec![0.5; buffer_size];
                let mut output = vec![0.0; buffer_size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark FFT for various sizes
fn bench_fft(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft");
    
    // Test power-of-2 sizes
    for size in [64, 128, 256, 512, 1024, 2048, 4096, 8192].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        // Forward FFT
        group.bench_with_input(
            BenchmarkId::new("forward", size),
            size,
            |b, &size| {
                let fft = FFT::new(size).unwrap();
                let input = vec![0.5; size];
                let mut output = vec![num_complex::Complex::new(0.0, 0.0); size];
                
                b.iter(|| {
                    fft.forward(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // Inverse FFT
        group.bench_with_input(
            BenchmarkId::new("inverse", size),
            size,
            |b, &size| {
                let fft = FFT::new(size).unwrap();
                let input = vec![num_complex::Complex::new(0.5, 0.0); size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    fft.inverse(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // Forward magnitude (frequency-only)
        group.bench_with_input(
            BenchmarkId::new("forward_magnitude", size),
            size,
            |b, &size| {
                let fft = FFT::new(size).unwrap();
                let input = vec![0.5; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    fft.forward_magnitude(black_box(&input), black_box(&mut output));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark FFT round-trip
fn bench_fft_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_roundtrip");
    
    for size in [256, 1024, 4096].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("size", size),
            size,
            |b, &size| {
                let fft = FFT::new(size).unwrap();
                let input = vec![0.5; size];
                let mut freq = vec![num_complex::Complex::new(0.0, 0.0); size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    fft.forward(black_box(&input), black_box(&mut freq));
                    fft.inverse(black_box(&freq), black_box(&mut output));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark processor chain
fn bench_processor_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("processor_chain");
    
    let buffer_size = 1024;
    group.throughput(Throughput::Elements(buffer_size as u64));
    
    // Single processor
    group.bench_function("single_processor", |b| {
        let mut chain = ProcessorChain::new();
        let mut gain = Gain::new();
        gain.set_gain_db(6.0);
        chain.add(gain);
        chain.prepare(44100.0, buffer_size);
        
        let input = vec![0.5; buffer_size];
        let mut output = vec![0.0; buffer_size];
        
        b.iter(|| {
            chain.process(black_box(&input), black_box(&mut output));
        });
    });
    
    // Two processors
    group.bench_function("two_processors", |b| {
        let mut chain = ProcessorChain::new();
        let mut gain = Gain::new();
        gain.set_gain_db(6.0);
        chain.add(gain);
        
        let mut bias = Bias::new();
        bias.set_bias(0.1);
        chain.add(bias);
        
        chain.prepare(44100.0, buffer_size);
        
        let input = vec![0.5; buffer_size];
        let mut output = vec![0.0; buffer_size];
        
        b.iter(|| {
            chain.process(black_box(&input), black_box(&mut output));
        });
    });
    
    // Complex chain (overdrive effect)
    group.bench_function("overdrive_chain", |b| {
        let mut chain = ProcessorChain::new();
        
        // Input gain
        let mut gain1 = Gain::new();
        gain1.set_gain_db(12.0);
        chain.add(gain1);
        
        // Bias for asymmetric distortion
        let mut bias = Bias::new();
        bias.set_bias(0.2);
        chain.add(bias);
        
        // Wave shaper
        let waveshaper = WaveShaper::new(transfer_functions::tanh);
        chain.add(waveshaper);
        
        // Output gain
        let mut gain2 = Gain::new();
        gain2.set_gain_db(-6.0);
        chain.add(gain2);
        
        chain.prepare(44100.0, buffer_size);
        
        let input = vec![0.5; buffer_size];
        let mut output = vec![0.0; buffer_size];
        
        b.iter(|| {
            chain.process(black_box(&input), black_box(&mut output));
        });
    });
    
    group.finish();
}

// Benchmark individual processors
fn bench_processors(c: &mut Criterion) {
    let mut group = c.benchmark_group("processors");
    
    let buffer_size = 1024;
    group.throughput(Throughput::Elements(buffer_size as u64));
    
    // Gain processor
    group.bench_function("gain", |b| {
        let mut gain = Gain::new();
        gain.prepare(44100.0, buffer_size);
        gain.set_gain_db(6.0);
        
        let input = vec![0.5; buffer_size];
        let mut output = vec![0.0; buffer_size];
        
        b.iter(|| {
            gain.process(black_box(&input), black_box(&mut output));
        });
    });
    
    // Bias processor
    group.bench_function("bias", |b| {
        let mut bias = Bias::new();
        bias.prepare(44100.0, buffer_size);
        bias.set_bias(0.1);
        
        let input = vec![0.5; buffer_size];
        let mut output = vec![0.0; buffer_size];
        
        b.iter(|| {
            bias.process(black_box(&input), black_box(&mut output));
        });
    });
    
    // Wave shaper (tanh)
    group.bench_function("waveshaper_tanh", |b| {
        let mut waveshaper = WaveShaper::new(transfer_functions::tanh);
        waveshaper.prepare(44100.0, buffer_size);
        
        let input = vec![0.5; buffer_size];
        let mut output = vec![0.0; buffer_size];
        
        b.iter(|| {
            waveshaper.process(black_box(&input), black_box(&mut output));
        });
    });
    
    // Wave shaper (hard clip)
    group.bench_function("waveshaper_hard_clip", |b| {
        let mut waveshaper = WaveShaper::new(transfer_functions::hard_clip);
        waveshaper.prepare(44100.0, buffer_size);
        
        let input = vec![0.5; buffer_size];
        let mut output = vec![0.0; buffer_size];
        
        b.iter(|| {
            waveshaper.process(black_box(&input), black_box(&mut output));
        });
    });
    
    group.finish();
}

// Benchmark SIMD vs scalar state variable filter
#[cfg(feature = "simd")]
fn bench_simd_state_variable_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_variable_filter_simd_vs_scalar");
    
    for buffer_size in [64, 256, 1024, 4096].iter() {
        group.throughput(Throughput::Elements(*buffer_size as u64));
        
        // Scalar version
        group.bench_with_input(
            BenchmarkId::new("scalar", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = StateVariableFilter::new();
                filter.prepare(44100.0).unwrap();
                filter.set_type(FilterType::Lowpass);
                filter.set_cutoff(1000.0);
                filter.set_resonance(0.7);
                
                let input = vec![0.5; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // SIMD version
        group.bench_with_input(
            BenchmarkId::new("simd", buffer_size),
            buffer_size,
            |b, &size| {
                let mut filter = SimdStateVariableFilter::new();
                filter.prepare(44100.0).unwrap();
                filter.set_type(FilterType::Lowpass);
                filter.set_cutoff(1000.0);
                filter.set_resonance(0.7);
                
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

// Benchmark SIMD vs scalar FIR filter
#[cfg(feature = "simd")]
fn bench_simd_fir_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("fir_filter_simd_vs_scalar");
    
    // Test with different filter lengths
    for filter_length in [16, 64, 128].iter() {
        let coeffs = FilterDesign::fir_lowpass(
            1000.0,
            44100.0,
            *filter_length,
            WindowFunction::Hamming,
        ).unwrap();
        
        let buffer_size = 1024;
        group.throughput(Throughput::Elements(buffer_size as u64));
        
        // Scalar version
        group.bench_with_input(
            BenchmarkId::new("scalar", filter_length),
            &coeffs,
            |b, coeffs| {
                let mut filter = FIRFilter::new(coeffs.clone());
                let input = vec![0.5; buffer_size];
                let mut output = vec![0.0; buffer_size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
        
        // SIMD version
        group.bench_with_input(
            BenchmarkId::new("simd", filter_length),
            &coeffs,
            |b, coeffs| {
                let mut filter = SimdFIRFilter::new(coeffs.clone());
                let input = vec![0.5; buffer_size];
                let mut output = vec![0.0; buffer_size];
                
                b.iter(|| {
                    filter.process(black_box(&input), black_box(&mut output));
                });
            },
        );
    }
    
    group.finish();
}

#[cfg(feature = "simd")]
criterion_group!(
    benches,
    bench_iir_filter_processing,
    bench_iir_filter_sample,
    bench_oscillator_generation,
    bench_oscillator_sample,
    bench_oscillator_frequency_modulation,
    bench_state_variable_filter,
    bench_state_variable_filter_sample,
    bench_fir_filter,
    bench_fir_filter_windows,
    bench_fft,
    bench_fft_roundtrip,
    bench_processor_chain,
    bench_processors,
    bench_simd_state_variable_filter,
    bench_simd_fir_filter,
);

#[cfg(not(feature = "simd"))]
criterion_group!(
    benches,
    bench_iir_filter_processing,
    bench_iir_filter_sample,
    bench_oscillator_generation,
    bench_oscillator_sample,
    bench_oscillator_frequency_modulation,
    bench_state_variable_filter,
    bench_state_variable_filter_sample,
    bench_fir_filter,
    bench_fir_filter_windows,
    bench_fft,
    bench_fft_roundtrip,
    bench_processor_chain,
    bench_processors,
);

criterion_main!(benches);
