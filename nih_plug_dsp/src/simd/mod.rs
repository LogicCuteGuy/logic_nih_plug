//! SIMD (Single Instruction Multiple Data) optimizations for DSP operations.
//!
//! This module provides vectorized implementations of DSP algorithms for improved
//! performance on modern CPUs. SIMD operations process multiple samples simultaneously,
//! significantly reducing processing time for filters, oscillators, and other operations.
//!
//! # Platform Support
//!
//! - **x86/x86_64**: SSE, AVX, AVX2 (runtime detection)
//! - **ARM**: NEON (compile-time detection)
//! - **Fallback**: Portable scalar implementations
//!
//! # Feature Flag
//!
//! This module is only available when the `simd` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! nih_plug_dsp = { version = "0.0.0", features = ["simd"] }
//! ```
//!
//! # Examples
//!
//! ```
//! use nih_plug_dsp::simd::optimizations::SimdStateVariableFilter;
//!
//! let mut filter = SimdStateVariableFilter::new();
//! filter.prepare(44100.0).unwrap();
//!
//! let input = vec![1.0; 1024];
//! let mut output = vec![0.0; 1024];
//! filter.process(&input, &mut output);
//! ```

pub mod optimizations;

// Re-export commonly used types
pub use optimizations::{SimdStateVariableFilter, SimdFIRFilter};
