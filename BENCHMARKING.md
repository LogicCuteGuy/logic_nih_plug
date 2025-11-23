# Benchmarking Suite

This document describes the benchmarking suite for the JUCE module ports and provides guidance on running benchmarks and interpreting results.

## Overview

The benchmarking suite measures the performance of core DSP operations, audio file I/O, and graphics primitives to ensure they meet performance requirements. All benchmarks are implemented using the [Criterion](https://github.com/bheisler/criterion.rs) benchmarking framework, which provides statistical analysis and regression detection.

## Running Benchmarks

### Run All Benchmarks

To run all benchmarks across all modules:

```bash
cargo bench
```

### Run Specific Module Benchmarks

To run benchmarks for a specific module:

```bash
# DSP benchmarks
cargo bench --package nih_plug_dsp

# Audio formats benchmarks
cargo bench --package nih_plug_audio_formats

# Graphics benchmarks
cargo bench --package nih_plug_graphics
```

### Run Specific Benchmark Groups

To run a specific benchmark group:

```bash
# IIR filter processing benchmarks
cargo bench --package nih_plug_dsp iir_filter_processing

# WAV file writing benchmarks
cargo bench --package nih_plug_audio_formats wav_write

# Rectangle filling benchmarks
cargo bench --package nih_plug_graphics fill_rect
```

### Baseline Comparisons

To save a baseline for comparison:

```bash
cargo bench --package nih_plug_dsp -- --save-baseline my-baseline
```

To compare against a saved baseline:

```bash
cargo bench --package nih_plug_dsp -- --baseline my-baseline
```

## Benchmark Organization

### DSP Benchmarks (`nih_plug_dsp`)

Located in `nih_plug_dsp/benches/dsp_benchmarks.rs`

#### IIR Filter Benchmarks

- **`iir_filter_processing`**: Measures filter processing performance at various buffer sizes (64-2048 samples)
  - Tests first-order, second-order, and third-order filters
  - Measures throughput in samples per second
  
- **`iir_filter_sample`**: Measures per-sample filter processing
  - Tests first-order and second-order filters
  - Useful for understanding single-sample latency

#### Oscillator Benchmarks

- **`oscillator_generation`**: Measures waveform generation at various buffer sizes (64-2048 samples)
  - Tests sine, saw, square, and triangle waveforms
  - Measures throughput in samples per second
  
- **`oscillator_sample`**: Measures per-sample oscillator generation
  - Tests all waveform types
  - Useful for understanding single-sample latency
  
- **`oscillator_frequency_modulation`**: Measures performance with per-sample frequency changes
  - Simulates FM synthesis scenarios
  - Tests phase continuity under modulation

### Audio I/O Benchmarks (`nih_plug_audio_formats`)

Located in `nih_plug_audio_formats/benches/audio_io_benchmarks.rs`

#### WAV File Benchmarks

- **`wav_write`**: Measures WAV file writing performance
  - Tests 16-bit, 24-bit, and 32-bit float formats
  - Tests various audio lengths (0.1s, 1s, 10s)
  - Measures throughput in samples per second
  
- **`wav_read`**: Measures WAV file reading performance
  - Tests 16-bit, 24-bit, and 32-bit float formats
  - Tests various audio lengths
  - Measures throughput in samples per second
  
- **`wav_roundtrip`**: Measures combined write + read performance
  - Tests all bit depths
  - Useful for understanding total I/O overhead
  
- **`wav_multichannel`**: Measures performance with different channel counts
  - Tests 1, 2, 4, 8, and 16 channels
  - Measures throughput scaling with channel count

### Graphics Benchmarks (`nih_plug_graphics`)

Located in `nih_plug_graphics/benches/graphics_benchmarks.rs`

#### Primitive Drawing Benchmarks

- **`set_pixel`**: Measures individual pixel setting performance
  - Tests single pixel and batch operations
  
- **`fill_rect`**: Measures rectangle filling at various sizes
  - Tests 10x10 to 400x400 rectangles
  - Measures throughput in pixels per second
  
- **`draw_line`**: Measures line drawing at various lengths
  - Tests horizontal, vertical, and diagonal lines
  - Uses Bresenham's algorithm
  
- **`draw_circle`**: Measures circle drawing at various radii
  - Tests radii from 5 to 200 pixels
  - Uses midpoint circle algorithm

#### Canvas Operations

- **`clear`**: Measures full canvas clearing at various resolutions
  - Tests 640x480 to 1920x1080
  - Measures throughput in pixels per second

#### Transformation Benchmarks

- **`transformations`**: Measures transformation operations
  - Tests translate, rotate, scale
  - Tests save/restore transform stack

#### Complex Scene Benchmarks

- **`complex_scene`**: Measures realistic drawing scenarios
  - Tests mixed primitive drawing
  - Tests drawing with transformations
  - Simulates typical UI rendering workloads

#### Buffer Access Benchmarks

- **`buffer_access`**: Measures pixel buffer access performance
  - Tests reading pixel data
  - Tests individual pixel queries

#### Context Creation

- **`context_creation`**: Measures graphics context allocation
  - Tests various canvas sizes
  - Useful for understanding initialization overhead

## Performance Targets

Based on the design document requirements, here are the performance targets:

### DSP Operations

| Operation | Buffer Size | Target Performance |
|-----------|-------------|-------------------|
| IIR Filter (1st order) | 1024 samples | < 10 μs |
| IIR Filter (2nd order) | 1024 samples | < 10 μs |
| Oscillator Generation | 1024 samples | < 5 μs |
| Per-sample Processing | 1 sample | < 50 ns |

### Audio File I/O

| Operation | Audio Length | Target Performance |
|-----------|--------------|-------------------|
| WAV Write (16-bit) | 1 second stereo | < 10 ms |
| WAV Read (16-bit) | 1 second stereo | < 10 ms |
| Round-trip | 1 second stereo | < 20 ms |

### Graphics Operations

| Operation | Size | Target Performance |
|-----------|------|-------------------|
| Fill Rectangle | 100x100 | < 100 μs |
| Draw Line | 100 pixels | < 10 μs |
| Draw Circle | radius 50 | < 50 μs |
| Clear Canvas | 800x600 | < 1 ms |

## Interpreting Results

### Understanding Criterion Output

Criterion provides several metrics for each benchmark:

- **Time**: The mean execution time with confidence intervals
- **Throughput**: Elements processed per second (when applicable)
- **Change**: Percentage change from previous run or baseline
- **Outliers**: Statistical outliers that may indicate measurement issues

Example output:

```
iir_filter_processing/first_order/1024
                        time:   [8.2341 μs 8.2891 μs 8.3512 μs]
                        thrpt:  [122.61 Melem/s 123.53 Melem/s 124.35 Melem/s]
                 change:
                        time:   [-2.1234% -1.5678% -0.9876%] (p = 0.00 < 0.05)
                        thrpt:  [+1.0000% +1.5923% +2.1765%]
                        Performance has improved.
```

### Performance Regression Detection

Criterion automatically detects performance regressions by comparing against previous runs. A regression is indicated when:

- The change percentage is significantly positive (slower)
- The p-value is less than 0.05 (statistically significant)

### Comparing Against Baselines

To track performance over time:

1. Save a baseline after implementing a feature:
   ```bash
   cargo bench -- --save-baseline feature-v1
   ```

2. Make changes and compare:
   ```bash
   cargo bench -- --baseline feature-v1
   ```

3. Review the comparison report in `target/criterion/`

## Performance Profiling

For detailed performance analysis, use profiling tools:

### Linux (perf)

```bash
cargo bench --package nih_plug_dsp --no-run
perf record --call-graph=dwarf target/release/deps/dsp_benchmarks-* --bench
perf report
```

### macOS (Instruments)

```bash
cargo bench --package nih_plug_dsp --no-run
instruments -t "Time Profiler" target/release/deps/dsp_benchmarks-*
```

### Windows (VTune or Visual Studio Profiler)

```bash
cargo bench --package nih_plug_dsp --no-run
# Use VTune or Visual Studio to profile the benchmark executable
```

## Continuous Integration

Benchmarks can be integrated into CI pipelines to detect performance regressions:

```yaml
# Example GitHub Actions workflow
- name: Run benchmarks
  run: cargo bench --all -- --save-baseline ci-baseline

- name: Compare against main
  run: cargo bench --all -- --baseline ci-baseline
```

## Optimization Guidelines

When optimizing based on benchmark results:

1. **Profile First**: Use profiling tools to identify bottlenecks
2. **Measure Impact**: Run benchmarks before and after changes
3. **Consider Trade-offs**: Balance performance with code clarity
4. **Test Correctness**: Ensure optimizations don't break functionality
5. **Document Changes**: Note performance improvements in commit messages

## Common Performance Issues

### DSP Operations

- **Denormal Numbers**: Can cause significant slowdowns in filters
  - Solution: Snap very small values to zero
  - Already implemented in `IIRFilter::snap_to_zero()`

- **Branch Prediction**: Conditional logic in tight loops
  - Solution: Use branchless algorithms where possible
  - Example: Optimized filter orders in `IIRFilter`

### Audio File I/O

- **Buffering**: Small read/write operations
  - Solution: Use buffered I/O (already implemented via `hound`)
  
- **Format Conversion**: Repeated conversions between formats
  - Solution: Batch conversions, use native format when possible

### Graphics Operations

- **Cache Misses**: Non-sequential memory access
  - Solution: Process pixels in scan-line order
  
- **Redundant Calculations**: Repeated transformation calculations
  - Solution: Cache transformation matrices

## Benchmark Maintenance

### Adding New Benchmarks

When adding new functionality:

1. Create a benchmark in the appropriate `benches/` directory
2. Follow the existing naming conventions
3. Include throughput measurements where applicable
4. Document expected performance in this file

### Updating Benchmarks

When modifying existing code:

1. Run benchmarks before changes to establish baseline
2. Run benchmarks after changes to measure impact
3. Update performance targets if necessary
4. Document significant changes in CHANGELOG.md

## Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)

## Troubleshooting

### Benchmarks Take Too Long

Reduce the number of samples or iterations:

```bash
cargo bench -- --sample-size 10
```

### Inconsistent Results

Ensure system is idle during benchmarking:

- Close unnecessary applications
- Disable CPU frequency scaling
- Run multiple times and compare

### Out of Memory

Reduce test data sizes in benchmarks or run benchmarks individually:

```bash
cargo bench --package nih_plug_dsp iir_filter_processing
```

## Contact

For questions about benchmarking or performance issues, please open an issue on the GitHub repository.
