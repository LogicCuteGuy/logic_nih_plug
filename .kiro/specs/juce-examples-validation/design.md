# Design Document

## Overview

This document provides a comprehensive design for validating and extending the ported JUCE modules based on analysis of the original JUCE examples. The design identifies missing features, proposes implementations, and defines correctness properties to ensure feature parity and correctness with the original JUCE framework.

The validation revealed several categories of missing functionality:
1. **Advanced DSP Components**: State variable filters, FIR filter design, wave shapers, processor chains
2. **Audio Analysis**: FFT support for frequency-domain processing
3. **Performance Optimizations**: SIMD support for vectorized operations
4. **GUI Layout**: FlexBox layout system for responsive interfaces
5. **Utility Processors**: Bias, gain, and DC filtering utilities

## Architecture

### Module Organization

The missing features will be integrated into existing ported modules:

```
nih_plug_dsp/
├── filters/
│   ├── iir.rs (existing)
│   ├── fir.rs (new)
│   ├── state_variable.rs (new)
│   └── design.rs (new - filter design utilities)
├── processors/
│   ├── gain.rs (new)
│   ├── bias.rs (new)
│   ├── waveshaper.rs (new)
│   ├── chain.rs (new)
│   └── dc_filter.rs (new)
├── analysis/
│   └── fft.rs (new)
└── simd/
    └── optimizations.rs (new - optional feature)

nih_plug_gui/
└── layout/
    ├── absolute.rs (existing)
    ├── flex.rs (existing - needs FlexBox)
    └── flexbox.rs (new)
```

### Design Principles

1. **API Consistency**: All new components follow existing nih-plug patterns
2. **Zero-Cost Abstractions**: Performance equivalent to hand-written code
3. **Type Safety**: Leverage Rust's type system to prevent misuse
4. **Composability**: Components can be easily combined and chained
5. **Testability**: All components designed for property-based testing

## Components and Interfaces

### State Variable Filter

```rust
pub struct StateVariableFilter {
    filter_type: FilterType,
    cutoff_hz: f32,
    resonance: f32,
    sample_rate: f32,
    // TPT state variables
    s1: f32,
    s2: f32,
}

pub enum FilterType {
    Lowpass,
    Bandpass,
    Highpass,
}

impl StateVariableFilter {
    pub fn new() -> Self;
    pub fn set_type(&mut self, filter_type: FilterType);
    pub fn set_cutoff(&mut self, hz: f32);
    pub fn set_resonance(&mut self, q: f32);
    pub fn process_sample(&mut self, input: f32) -> f32;
    pub fn process(&mut self, input: &[f32], output: &mut [f32]);
    pub fn reset(&mut self);
}
```

### FIR Filter and Design

```rust
pub struct FIRFilter {
    coefficients: Vec<f32>,
    delay_line: Vec<f32>,
    write_pos: usize,
}

pub enum WindowFunction {
    Rectangular,
    Triangular,
    Hann,
    Hamming,
    Blackman,
    BlackmanHarris,
    FlatTop,
    Kaiser { beta: f32 },
}

pub struct FilterDesign;

impl FilterDesign {
    pub fn fir_lowpass(
        cutoff_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Vec<f32>;
    
    pub fn fir_highpass(
        cutoff_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Vec<f32>;
    
    pub fn fir_bandpass(
        low_hz: f32,
        high_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Vec<f32>;
    
    pub fn fir_bandstop(
        low_hz: f32,
        high_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Vec<f32>;
}
```

### Wave Shaper

```rust
pub struct WaveShaper<F>
where
    F: Fn(f32) -> f32,
{
    transfer_function: F,
}

impl<F> WaveShaper<F>
where
    F: Fn(f32) -> f32,
{
    pub fn new(transfer_function: F) -> Self;
    pub fn process_sample(&self, input: f32) -> f32;
    pub fn process(&self, input: &[f32], output: &mut [f32]);
}

// Predefined transfer functions
pub mod transfer_functions {
    pub fn tanh(x: f32) -> f32;
    pub fn tanh_approx(x: f32) -> f32;  // Fast approximation
    pub fn hard_clip(x: f32) -> f32;
    pub fn soft_clip(x: f32) -> f32;
}
```

### Processor Chain

```rust
pub trait Processor {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize);
    fn process(&mut self, input: &[f32], output: &mut [f32]);
    fn reset(&mut self);
}

pub struct ProcessorChain {
    processors: Vec<Box<dyn Processor>>,
}

impl ProcessorChain {
    pub fn new() -> Self;
    pub fn add<P: Processor + 'static>(&mut self, processor: P);
    pub fn get(&self, index: usize) -> Option<&dyn Processor>;
    pub fn get_mut(&mut self, index: usize) -> Option<&mut dyn Processor>;
    pub fn len(&self) -> usize;
}

impl Processor for ProcessorChain {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize);
    fn process(&mut self, input: &[f32], output: &mut [f32]);
    fn reset(&mut self);
}
```

### Gain and Bias Processors

```rust
pub struct Gain {
    gain_linear: f32,
    smoothed_gain: f32,
    smoothing_coeff: f32,
}

impl Gain {
    pub fn new() -> Self;
    pub fn set_gain_db(&mut self, db: f32);
    pub fn set_gain_linear(&mut self, gain: f32);
    pub fn set_smoothing_time(&mut self, time_ms: f32, sample_rate: f32);
}

pub struct Bias {
    offset: f32,
}

impl Bias {
    pub fn new() -> Self;
    pub fn set_bias(&mut self, offset: f32);
}
```

### FFT Analysis

```rust
pub struct FFT {
    size: usize,
    fft_impl: Arc<dyn rustfft::Fft<f32>>,
}

impl FFT {
    pub fn new(size: usize) -> Result<Self, DspError>;
    pub fn forward(&self, input: &[f32], output: &mut [Complex<f32>]);
    pub fn inverse(&self, input: &[Complex<f32>], output: &mut [f32]);
    pub fn forward_magnitude(&self, input: &[f32], output: &mut [f32]);
}
```

### FlexBox Layout

```rust
pub struct FlexBox {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub items: Vec<FlexItem>,
}

pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
}

pub struct FlexItem {
    pub order: i32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    pub align_self: AlignSelf,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub margin: Margin,
}

impl FlexBox {
    pub fn new() -> Self;
    pub fn add_item(&mut self, item: FlexItem);
    pub fn layout(&self, container_width: f32, container_height: f32) -> Vec<Rect>;
}
```

## Data Models

### Filter Coefficients

```rust
pub struct FIRCoefficients {
    pub coefficients: Vec<f32>,
    pub length: usize,
}

pub struct IIRCoefficients {
    pub numerator: Vec<f32>,
    pub denominator: Vec<f32>,
}
```

### Layout Geometry

```rust
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### State Variable Filter Properties

**Property 1: Filter type switching maintains state continuity**
*For any* state variable filter with existing state, when the filter type is changed, processing the next sample should not produce a discontinuity greater than the expected filter response
**Validates: Requirements 1.2**

**Property 2: TPT filter stability**
*For any* cutoff frequency, resonance value, and input signal, the state variable filter output should remain finite and not produce NaN or infinity
**Validates: Requirements 1.3, 1.4**

**Property 3: Reset preserves coefficients**
*For any* state variable filter with set parameters, resetting the filter and then processing the same input should produce the same output as a freshly created filter with the same parameters
**Validates: Requirements 1.5**

### FIR Filter Properties

**Property 4: Window function diversity**
*For any* two different window functions applied to the same filter specification, the resulting FIR coefficients should be different
**Validates: Requirements 2.1**

**Property 5: FIR frequency response accuracy**
*For any* FIR lowpass filter designed with cutoff frequency fc, the magnitude response at fc should be approximately -3dB (within 1dB tolerance)
**Validates: Requirements 2.2**

**Property 6: FIR linear phase**
*For any* FIR filter, the group delay should be constant across all frequencies (within numerical precision)
**Validates: Requirements 2.3**

**Property 7: Nyquist validation**
*For any* filter design with cutoff frequency >= Nyquist frequency, the system should return an error or clamp the cutoff to valid range
**Validates: Requirements 9.3**

### Wave Shaper Properties

**Property 8: Transfer function application**
*For any* input sample x and transfer function f, the wave shaper output should equal f(x)
**Validates: Requirements 3.2**

**Property 9: Processor chain composition**
*For any* sequence of processors [P1, P2, ..., Pn] and input signal, processing through a chain should produce the same output as applying each processor sequentially
**Validates: Requirements 3.4, 4.2**

### Bias Processor Properties

**Property 10: Bias addition**
*For any* input signal and bias value b, the output should equal input + b for all samples
**Validates: Requirements 5.1**

**Property 11: Bias numerical stability**
*For any* finite input signal and finite bias value, the output should remain finite
**Validates: Requirements 5.3**

### Processor Chain Properties

**Property 12: Chain preparation propagation**
*For any* processor chain, after calling prepare(), all processors in the chain should be in prepared state
**Validates: Requirements 4.4**

**Property 13: Chain reset propagation**
*For any* processor chain, after calling reset(), all processors should be in reset state
**Validates: Requirements 4.5**

### FFT Properties

**Property 14: FFT round-trip**
*For any* input signal, performing forward FFT followed by inverse FFT should reconstruct the original signal (within numerical precision tolerance of 1e-5)
**Validates: Requirements 6.2, 6.3**

**Property 15: FFT magnitude spectrum**
*For any* input signal, the frequency-only transform should produce non-negative magnitude values
**Validates: Requirements 6.4**

**Property 16: FFT power-of-2 sizes**
*For any* power-of-2 size from 2 to 65536, FFT creation should succeed
**Validates: Requirements 6.1**

### SIMD Properties

**Property 17: SIMD equivalence**
*For any* input signal, processing with SIMD-optimized code should produce identical results to scalar code (within floating-point precision)
**Validates: Requirements 7.2, 7.3**

### FlexBox Layout Properties

**Property 18: FlexBox direction consistency**
*For any* set of flex items, changing flex-direction should reorder items according to CSS FlexBox specification
**Validates: Requirements 8.1**

**Property 19: FlexBox wrapping behavior**
*For any* set of items that exceed container width, wrap mode should cause items to flow to next line
**Validates: Requirements 8.2**

**Property 20: FlexBox justify-content spacing**
*For any* set of items with space-between justification, the space between adjacent items should be equal
**Validates: Requirements 8.3**

**Property 21: FlexBox align-self override**
*For any* flex item with align-self set, its alignment should differ from container align-items setting
**Validates: Requirements 8.5**

### Gain Processor Properties

**Property 22: Decibel conversion accuracy**
*For any* gain value in dB, converting to linear gain and back should preserve the original value (within 0.01 dB)
**Validates: Requirements 12.2**

**Property 23: Gain application**
*For any* input signal and linear gain g, the output should equal input * g for all samples (after smoothing settles)
**Validates: Requirements 12.3**

**Property 24: Gain smoothing continuity**
*For any* gain change, the output signal should not contain discontinuities larger than the smoothing step size
**Validates: Requirements 12.4**

### DC Filter Properties

**Property 25: DC removal**
*For any* input signal with DC offset, processing through DC filter should reduce DC component to near zero while preserving AC components above 20Hz
**Validates: Requirements 11.2**

**Property 26: DC filter sample rate adaptation**
*For any* two sample rates, the DC filter cutoff frequency in Hz should remain constant when sample rate changes
**Validates: Requirements 11.3, 11.5**

### Filter Design Properties

**Property 27: Filter design numerical stability**
*For any* valid filter specification, the designed filter coefficients should not cause instability (all poles inside unit circle for IIR)
**Validates: Requirements 9.4**

## Error Handling

### Error Types

```rust
pub enum DspError {
    InvalidParameter { param: String, value: f32, reason: String },
    InvalidFilterLength { length: usize, min: usize, max: usize },
    InvalidFFTSize { size: usize },
    NumericalInstability { component: String },
    NotPrepared { component: String },
}
```

### Error Scenarios

1. **Invalid Parameters**: Cutoff frequency outside valid range, negative resonance, etc.
2. **Invalid Filter Length**: FIR filter length too short or not odd
3. **Invalid FFT Size**: Non-power-of-2 FFT size
4. **Numerical Instability**: Filter coefficients would cause instability
5. **Not Prepared**: Attempting to process before calling prepare()

## Testing Strategy

### Unit Tests

1. **Component Creation**: Test that all components can be created with default parameters
2. **Parameter Validation**: Test that invalid parameters are rejected
3. **Edge Cases**: Test with zero input, maximum values, boundary conditions
4. **State Management**: Test prepare/reset cycles

### Property-Based Tests

Property-based tests will use the `proptest` crate to verify correctness properties across thousands of randomly generated inputs:

1. **Filter Stability**: Generate random filter parameters and verify no NaN/infinity
2. **Round-Trip Properties**: FFT/IFFT, serialize/deserialize
3. **Mathematical Properties**: Gain conversion, bias addition, transfer functions
4. **Layout Properties**: FlexBox spacing, alignment, wrapping

### Integration Tests

1. **Processor Chains**: Test complex chains of multiple processors
2. **Real-World Scenarios**: Test with actual audio files and realistic parameters
3. **Performance**: Benchmark against JUCE implementations
4. **Cross-Platform**: Verify behavior on Windows, macOS, Linux

### Comparison Tests

1. **JUCE Parity**: Compare outputs with JUCE for identical inputs
2. **Example Validation**: Verify ported examples produce expected results
3. **Feature Completeness**: Ensure all JUCE example features are available

## Performance Considerations

### Optimization Strategies

1. **SIMD Vectorization**: Use SIMD for filters, oscillators, and basic operations
2. **Cache Efficiency**: Optimize data layout for cache locality
3. **Branch Prediction**: Minimize branches in hot loops
4. **Inlining**: Mark hot functions for inlining
5. **Zero-Copy**: Avoid unnecessary allocations in audio thread

### Performance Targets

| Operation | Target Performance |
|-----------|-------------------|
| State Variable Filter (1024 samples) | < 10 μs |
| FIR Filter (1024 samples, 64 taps) | < 15 μs |
| Wave Shaper (1024 samples) | < 3 μs |
| FFT (1024 points) | < 50 μs |
| FlexBox Layout (10 items) | < 100 μs |

## Implementation Notes

### State Variable Filter (TPT)

The Topology-Preserving Transform (TPT) method ensures stability at all parameter settings by using a trapezoidal integrator structure. The implementation follows:

```
g = tan(π * cutoff / sample_rate)
k = 2 - 2 * resonance
a1 = 1 / (1 + g * (g + k))
a2 = g * a1
a3 = g * a2

v0 = input
v1 = a1 * s1 + a2 * (v0 - s2)
v2 = s2 + a2 * s1 + a3 * (v0 - s2)

s1 = 2 * v1 - s1
s2 = 2 * v2 - s2

lowpass = v2
bandpass = v1
highpass = v0 - k * v1 - v2
```

### FIR Filter Design

Window-based FIR design uses the following approach:

1. Generate ideal sinc function impulse response
2. Apply window function to truncate and reduce ripple
3. Normalize coefficients for unity gain at DC (lowpass) or Nyquist (highpass)

### SIMD Implementation

SIMD optimizations will be feature-gated and use platform-specific intrinsics:

- x86/x86_64: SSE, AVX, AVX2
- ARM: NEON
- Fallback: Portable SIMD or scalar code

### FlexBox Algorithm

The FlexBox layout algorithm follows the CSS FlexBox specification:

1. Determine main and cross axes based on flex-direction
2. Calculate available space
3. Resolve flexible lengths (flex-grow/flex-shrink)
4. Distribute space according to justify-content
5. Align items on cross axis according to align-items/align-self
6. Handle multi-line layouts with align-content

## Migration Path

### For Existing Code

1. **State Variable Filters**: Replace custom filter implementations with `StateVariableFilter`
2. **Distortion Effects**: Use `WaveShaper` with appropriate transfer functions
3. **Effect Chains**: Migrate to `ProcessorChain` for cleaner code
4. **Spectrum Analysis**: Use `FFT` for frequency-domain processing
5. **Responsive UI**: Adopt `FlexBox` for adaptive layouts

### API Compatibility

All new components follow existing nih-plug patterns:
- `prepare()` for initialization
- `process()` for audio processing
- `reset()` for state clearing
- Builder patterns for configuration
- `Result<T, E>` for error handling

## Future Enhancements

1. **Additional Filter Types**: Elliptic, Chebyshev, Bessel filters
2. **More Transfer Functions**: Polynomial, lookup table-based
3. **Advanced FFT**: Overlap-add, STFT with perfect reconstruction
4. **GPU Acceleration**: Compute shader-based processing
5. **Visual Layout Editor**: GUI tool for FlexBox layout design
