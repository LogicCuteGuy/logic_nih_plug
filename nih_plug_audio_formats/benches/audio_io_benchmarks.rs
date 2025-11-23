//! Benchmarks for audio file I/O operations.
//!
//! This benchmark suite measures the performance of audio file reading and writing
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nih_plug_audio_formats::wav::{WavReader, WavWriter};
use std::fs;

// Helper function to create test audio data
fn create_test_audio(num_channels: usize, num_frames: usize) -> Vec<Vec<f32>> {
    (0..num_channels)
        .map(|ch| {
            (0..num_frames)
                .map(|i| {
                    let phase = (i as f32 / num_frames as f32) * 2.0 * std::f32::consts::PI;
                    (phase * (ch + 1) as f32).sin() * 0.5
                })
                .collect()
        })
        .collect()
}

// Benchmark WAV file writing at different sizes and bit depths
fn bench_wav_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("wav_write");
    
    // Test different audio lengths (in seconds at 44.1kHz)
    let test_cases = vec![
        ("0.1s", 4410),   // 0.1 seconds
        ("1s", 44100),    // 1 second
        ("10s", 441000),  // 10 seconds
    ];
    
    for (name, num_frames) in test_cases {
        let stereo_data = create_test_audio(2, num_frames);
        let total_samples = num_frames * 2;
        
        group.throughput(Throughput::Elements(total_samples as u64));
        
        // 16-bit
        group.bench_with_input(
            BenchmarkId::new("16bit", name),
            &stereo_data,
            |b, data| {
                b.iter(|| {
                    let temp_file = format!("bench_wav_16_{}.wav", name);
                    let mut writer = WavWriter::create(&temp_file, 44100.0, 2, 16).unwrap();
                    writer.write_samples(black_box(data)).unwrap();
                    writer.finalize().unwrap();
                    fs::remove_file(temp_file).ok();
                });
            },
        );
        
        // 24-bit
        group.bench_with_input(
            BenchmarkId::new("24bit", name),
            &stereo_data,
            |b, data| {
                b.iter(|| {
                    let temp_file = format!("bench_wav_24_{}.wav", name);
                    let mut writer = WavWriter::create(&temp_file, 44100.0, 2, 24).unwrap();
                    writer.write_samples(black_box(data)).unwrap();
                    writer.finalize().unwrap();
                    fs::remove_file(temp_file).ok();
                });
            },
        );
        
        // 32-bit float
        group.bench_with_input(
            BenchmarkId::new("32bit_float", name),
            &stereo_data,
            |b, data| {
                b.iter(|| {
                    let temp_file = format!("bench_wav_32_{}.wav", name);
                    let mut writer = WavWriter::create(&temp_file, 44100.0, 2, 32).unwrap();
                    writer.write_samples(black_box(data)).unwrap();
                    writer.finalize().unwrap();
                    fs::remove_file(temp_file).ok();
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark WAV file reading at different sizes and bit depths
fn bench_wav_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("wav_read");
    
    // Create test files
    let test_cases = vec![
        ("0.1s", 4410),
        ("1s", 44100),
        ("10s", 441000),
    ];
    
    // Create test files for each case
    for (name, num_frames) in &test_cases {
        let data = create_test_audio(2, *num_frames);
        
        // 16-bit
        let file_16 = format!("bench_read_16_{}.wav", name);
        let mut writer = WavWriter::create(&file_16, 44100.0, 2, 16).unwrap();
        writer.write_samples(&data).unwrap();
        writer.finalize().unwrap();
        
        // 24-bit
        let file_24 = format!("bench_read_24_{}.wav", name);
        let mut writer = WavWriter::create(&file_24, 44100.0, 2, 24).unwrap();
        writer.write_samples(&data).unwrap();
        writer.finalize().unwrap();
        
        // 32-bit float
        let file_32 = format!("bench_read_32_{}.wav", name);
        let mut writer = WavWriter::create(&file_32, 44100.0, 2, 32).unwrap();
        writer.write_samples(&data).unwrap();
        writer.finalize().unwrap();
    }
    
    // Benchmark reading
    for (name, num_frames) in &test_cases {
        let total_samples = num_frames * 2;
        group.throughput(Throughput::Elements(total_samples as u64));
        
        // 16-bit
        let file_16 = format!("bench_read_16_{}.wav", name);
        group.bench_with_input(
            BenchmarkId::new("16bit", name),
            &file_16,
            |b, file| {
                b.iter(|| {
                    let mut reader = WavReader::open(black_box(file)).unwrap();
                    let _samples = reader.read_all().unwrap();
                });
            },
        );
        
        // 24-bit
        let file_24 = format!("bench_read_24_{}.wav", name);
        group.bench_with_input(
            BenchmarkId::new("24bit", name),
            &file_24,
            |b, file| {
                b.iter(|| {
                    let mut reader = WavReader::open(black_box(file)).unwrap();
                    let _samples = reader.read_all().unwrap();
                });
            },
        );
        
        // 32-bit float
        let file_32 = format!("bench_read_32_{}.wav", name);
        group.bench_with_input(
            BenchmarkId::new("32bit_float", name),
            &file_32,
            |b, file| {
                b.iter(|| {
                    let mut reader = WavReader::open(black_box(file)).unwrap();
                    let _samples = reader.read_all().unwrap();
                });
            },
        );
    }
    
    // Cleanup
    for (name, _) in &test_cases {
        fs::remove_file(format!("bench_read_16_{}.wav", name)).ok();
        fs::remove_file(format!("bench_read_24_{}.wav", name)).ok();
        fs::remove_file(format!("bench_read_32_{}.wav", name)).ok();
    }
    
    group.finish();
}

// Benchmark round-trip (write + read)
fn bench_wav_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("wav_roundtrip");
    
    let data = create_test_audio(2, 44100); // 1 second stereo
    group.throughput(Throughput::Elements(44100 * 2));
    
    group.bench_function("16bit", |b| {
        b.iter(|| {
            let temp_file = "bench_roundtrip_16.wav";
            
            // Write
            let mut writer = WavWriter::create(temp_file, 44100.0, 2, 16).unwrap();
            writer.write_samples(black_box(&data)).unwrap();
            writer.finalize().unwrap();
            
            // Read
            let mut reader = WavReader::open(temp_file).unwrap();
            let _samples = reader.read_all().unwrap();
            
            fs::remove_file(temp_file).ok();
        });
    });
    
    group.bench_function("24bit", |b| {
        b.iter(|| {
            let temp_file = "bench_roundtrip_24.wav";
            
            // Write
            let mut writer = WavWriter::create(temp_file, 44100.0, 2, 24).unwrap();
            writer.write_samples(black_box(&data)).unwrap();
            writer.finalize().unwrap();
            
            // Read
            let mut reader = WavReader::open(temp_file).unwrap();
            let _samples = reader.read_all().unwrap();
            
            fs::remove_file(temp_file).ok();
        });
    });
    
    group.bench_function("32bit_float", |b| {
        b.iter(|| {
            let temp_file = "bench_roundtrip_32.wav";
            
            // Write
            let mut writer = WavWriter::create(temp_file, 44100.0, 2, 32).unwrap();
            writer.write_samples(black_box(&data)).unwrap();
            writer.finalize().unwrap();
            
            // Read
            let mut reader = WavReader::open(temp_file).unwrap();
            let _samples = reader.read_all().unwrap();
            
            fs::remove_file(temp_file).ok();
        });
    });
    
    group.finish();
}

// Benchmark multi-channel audio
fn bench_wav_multichannel(c: &mut Criterion) {
    let mut group = c.benchmark_group("wav_multichannel");
    
    let num_frames = 44100; // 1 second
    
    for num_channels in [1, 2, 4, 8, 16].iter() {
        let data = create_test_audio(*num_channels, num_frames);
        let total_samples = num_frames * num_channels;
        
        group.throughput(Throughput::Elements(total_samples as u64));
        
        group.bench_with_input(
            BenchmarkId::new("write", num_channels),
            &data,
            |b, data| {
                b.iter(|| {
                    let temp_file = format!("bench_multichannel_{}.wav", num_channels);
                    let mut writer = WavWriter::create(&temp_file, 44100.0, *num_channels, 16).unwrap();
                    writer.write_samples(black_box(data)).unwrap();
                    writer.finalize().unwrap();
                    fs::remove_file(temp_file).ok();
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_wav_write,
    bench_wav_read,
    bench_wav_roundtrip,
    bench_wav_multichannel,
);
criterion_main!(benches);
