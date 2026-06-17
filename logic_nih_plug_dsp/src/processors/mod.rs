//! Audio processors for signal manipulation.
//!
//! This module contains various audio processors that can be chained together
//! to create complex audio processing pipelines.

pub mod bias;
pub mod chain;
pub mod dc_filter;
pub mod gain;
pub mod waveshaper;

// Re-export the Processor trait for convenience
pub use chain::Processor;
