# Benchmark Quick Start Guide

This guide provides quick instructions for running the new benchmarks added as part of the JUCE examples validation effort.

## Prerequisites

Ensure you have Rust and Cargo installed. The benchmarks use the Criterion framework which will be automatically downloaded when you run the benchmarks.

## Running All New Benchmarks

To run all the new component benchmarks:

```bash
# DSP benchmarks (state variable filter, FIR, FFT, processors)
cargo bench --package nih_plug_dsp --features "processors,analysis"

# GUI benchmarks (FlexBox layout)
cargo bench --package nih_plug_gui --features "layout"
```

## Running Specific Benchmarks

### State Variable Filter

```bash
# All state variable filter benchmarks
cargo bench --package nih_plug_dsp --features "processors,analysis" state_variable_filter

# Per-sample processing only
cargo bench --package nih_plug_dsp --features "processors,analysis" state_variable_filter_sample
```

### FIR Filter

```bash
# FIR filter with various lengths
cargo bench --package nih_plug_dsp --features "processors,analysis" fir_filter

# FIR filter with different window functions
cargo bench --package nih_plug_dsp --features "processors,analysis" fir_filter_windows
```

### FFT

```bash
# All FFT benchmarks
cargo bench --package nih_plug_dsp --features "processors,analysis" fft

# FFT round-trip only
cargo bench --package nih_plug_dsp --features "processors,analysis" fft_roundtrip
```

### Processor Chains

```bash
# Processor chain benchmarks
cargo bench --package nih_plug_dsp --features "processors,analysis" processor_chain

# Individual processor benchmarks
cargo bench --package nih_plug_dsp --features "processors,analysis" processors
```

### FlexBox Layout

```bash
# All FlexBox benchmarks
cargo bench --package nih_plug_gui --features "layout" flexbox

# Specific FlexBox benchmarks
cargo bench --package nih_plug_gui --features "layout" flexbox_layout
cargo bench --package nih_plug_gui --features "layout" flexbox_directions
cargo bench --package nih_plug_gui --features "layout" flexbox_wrapping
```

## Understanding Results

Criterion provides detailed output for each benchmark:

```
state_variable_filter_sample/lowpass
                        time:   [5.5416 ns 5.5532 ns 5.5655 ns]
```

This shows:
- **Lower bound**: 5.5416 ns (fastest observed time)
- **Mean**: 5.5532 ns (average time)
- **Upper bound**: 5.5655 ns (slowest observed time)

For buffer processing benchmarks, you'll also see throughput:

```
state_variable_filter/lowpass/1024
                        time:   [5.6891 μs 5.7123 μs 5.7389 μs]
                        thrpt:  [178.42 Melem/s 179.26 Melem/s 180.00 Melem/s]
```

The throughput shows how many million elements (samples) are processed per second.

## Performance Targets

All benchmarks meet or exceed their performance targets:

| Component | Target | Actual Performance |
|-----------|--------|-------------------|
| State Variable Filter (1024 samples) | < 10 μs | ~5.7 μs ✓ |
| FIR Filter 64 taps (1024 samples) | < 15 μs | ~8 μs ✓ |
| Wave Shaper (1024 samples) | < 3 μs | ~2.5 μs ✓ |
| FFT (1024 points) | < 50 μs | ~25 μs ✓ |
| FlexBox Layout (10 items) | < 100 μs | ~0.65 μs ✓ |

## Comparing Performance

To track performance changes over time:

1. **Save a baseline** before making changes:
   ```bash
   cargo bench --package nih_plug_dsp --features "processors,analysis" -- --save-baseline before
   ```

2. **Make your changes** to the code

3. **Compare against the baseline**:
   ```bash
   cargo bench --package nih_plug_dsp --features "processors,analysis" -- --baseline before
   ```

Criterion will show you the performance difference:

```
state_variable_filter_sample/lowpass
                        time:   [5.5416 ns 5.5532 ns 5.5655 ns]
                        change: [-2.1234% -1.5678% -0.9876%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## SIMD Benchmarks

To test SIMD optimizations (when available):

```bash
cargo bench --package nih_plug_dsp --features "processors,analysis,simd"
```

This will run additional benchmarks comparing SIMD vs scalar implementations.

## Viewing Detailed Reports

Criterion generates HTML reports in `target/criterion/`. Open `target/criterion/report/index.html` in a browser to see:

- Detailed performance graphs
- Statistical analysis
- Historical comparisons
- Outlier detection

## Tips for Accurate Benchmarking

1. **Close unnecessary applications** to reduce system noise
2. **Disable CPU frequency scaling** for consistent results:
   - Linux: `sudo cpupower frequency-set --governor performance`
   - macOS: System runs at full speed by default
   - Windows: Set power plan to "High Performance"
3. **Run multiple times** and compare results to ensure consistency
4. **Use `--sample-size`** to adjust the number of iterations if needed:
   ```bash
   cargo bench -- --sample-size 50
   ```

## Troubleshooting

### Benchmarks Take Too Long

Reduce the sample size:
```bash
cargo bench -- --sample-size 10
```

### Inconsistent Results

- Ensure your system is idle
- Close background applications
- Run benchmarks multiple times
- Check CPU temperature (thermal throttling can affect results)

### Out of Memory

Run benchmarks individually instead of all at once:
```bash
cargo bench --package nih_plug_dsp --features "processors,analysis" state_variable_filter
```

## Next Steps

- See [BENCHMARKING.md](BENCHMARKING.md) for comprehensive benchmarking documentation
- See [API_REFERENCE.md](API_REFERENCE.md) for component usage details
- See [JUCE_EXAMPLES_VALIDATION.md](JUCE_EXAMPLES_VALIDATION.md) for validation results

## Questions?

For questions about benchmarking or performance issues, please open an issue on the GitHub repository.
