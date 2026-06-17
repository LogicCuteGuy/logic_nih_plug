//! FFT-based convolution engine for reverb and impulse-response-based effects.
//!
//! This module provides efficient convolution using the overlap-add method with FFT.
//! It supports both mono and stereo processing, impulse response loading, and
//! crossfading when changing impulse responses.

use crate::error::DspError;
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Arc;

/// FFT-based convolution processor.
///
/// This processor uses the overlap-add method with FFT for efficient convolution.
/// It supports loading impulse responses, sample rate validation, and crossfading
/// when changing impulse responses to prevent clicks.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::convolution::Convolution;
///
/// let mut conv = Convolution::new(44100.0, 512);
///
/// // Load an impulse response
/// let ir = vec![1.0, 0.5, 0.25, 0.125];
/// conv.load_impulse_response(&ir, 44100.0).unwrap();
///
/// // Process audio
/// let input = vec![1.0; 512];
/// let mut output = vec![0.0; 512];
/// conv.process(&input, &mut output);
/// ```
///
/// # Performance
///
/// Processing time depends on the impulse response length and block size.
/// Longer impulse responses require larger FFT sizes and more computation.
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance.
pub struct Convolution {
    sample_rate: f32,
    block_size: usize,
    
    // Current impulse response (frequency domain)
    ir_fft: Vec<Complex<f32>>,
    ir_length: usize,
    
    // New impulse response for crossfading
    new_ir_fft: Option<Vec<Complex<f32>>>,
    new_ir_length: usize,
    crossfade_samples: usize,
    crossfade_position: usize,
    
    // FFT processing
    fft_size: usize,
    fft: Arc<dyn rustfft::Fft<f32>>,
    ifft: Arc<dyn rustfft::Fft<f32>>,
    
    // Overlap-add buffers
    input_buffer: Vec<Complex<f32>>,
    output_buffer: Vec<Complex<f32>>,
    overlap: Vec<f32>,
    
    // Processing state
    position: usize,
}

impl Convolution {
    /// Creates a new convolution processor.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate in Hz
    /// * `block_size` - The processing block size in samples
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::convolution::Convolution;
    ///
    /// let conv = Convolution::new(44100.0, 512);
    /// ```
    pub fn new(sample_rate: f32, block_size: usize) -> Self {
        if sample_rate <= 0.0 {
            panic!("Sample rate must be positive");
        }
        if block_size == 0 {
            panic!("Block size must be positive");
        }
        
        // Start with a minimal FFT size
        let fft_size = block_size.next_power_of_two() * 2;
        
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let ifft = planner.plan_fft_inverse(fft_size);
        
        Self {
            sample_rate,
            block_size,
            ir_fft: vec![Complex::new(0.0, 0.0); fft_size],
            ir_length: 0,
            new_ir_fft: None,
            new_ir_length: 0,
            crossfade_samples: (sample_rate * 0.05) as usize, // 50ms crossfade
            crossfade_position: 0,
            fft_size,
            fft,
            ifft,
            input_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            output_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            overlap: vec![0.0; fft_size],
            position: 0,
        }
    }
    
    /// Loads an impulse response from raw sample data.
    ///
    /// The impulse response will be validated against the processor's sample rate.
    /// If a different impulse response is already loaded, crossfading will be used
    /// to prevent clicks.
    ///
    /// # Arguments
    ///
    /// * `ir` - The impulse response samples
    /// * `sample_rate` - The sample rate of the impulse response
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The sample rate doesn't match the processor's sample rate
    /// - The impulse response is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::convolution::Convolution;
    ///
    /// let mut conv = Convolution::new(44100.0, 512);
    /// let ir = vec![1.0, 0.5, 0.25, 0.125];
    /// conv.load_impulse_response(&ir, 44100.0).unwrap();
    /// ```
    pub fn load_impulse_response(&mut self, ir: &[f32], sample_rate: f32) -> Result<(), DspError> {
        // Validate sample rate
        if (sample_rate - self.sample_rate).abs() > 0.1 {
            return Err(DspError::InvalidSampleRate(sample_rate));
        }
        
        // Validate impulse response
        if ir.is_empty() {
            return Err(DspError::InvalidBufferSize(0));
        }
        
        // Calculate required FFT size
        let required_fft_size = (self.block_size + ir.len()).next_power_of_two();
        
        // Resize FFT if needed
        if required_fft_size != self.fft_size {
            self.resize_fft(required_fft_size);
        }
        
        // Compute FFT of impulse response
        let mut ir_fft = vec![Complex::new(0.0, 0.0); self.fft_size];
        for (i, &sample) in ir.iter().enumerate() {
            ir_fft[i] = Complex::new(sample, 0.0);
        }
        self.fft.process(&mut ir_fft);
        
        // If we already have an IR loaded, prepare for crossfade
        if self.ir_length > 0 {
            self.new_ir_fft = Some(ir_fft);
            self.new_ir_length = ir.len();
            self.crossfade_position = 0;
        } else {
            // First IR load, no crossfade needed
            self.ir_fft = ir_fft;
            self.ir_length = ir.len();
        }
        
        Ok(())
    }
    
    /// Processes a block of audio through the convolution.
    ///
    /// The input and output slices must have the same length.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio samples
    /// * `output` - Output audio samples (will be overwritten)
    ///
    /// # Panics
    ///
    /// Panics if input and output have different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::convolution::Convolution;
    ///
    /// let mut conv = Convolution::new(44100.0, 512);
    /// let ir = vec![1.0, 0.5, 0.25];
    /// conv.load_impulse_response(&ir, 44100.0).unwrap();
    ///
    /// let input = vec![1.0; 512];
    /// let mut output = vec![0.0; 512];
    /// conv.process(&input, &mut output);
    /// ```
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len(), "Input and output must have same length");
        
        // If no IR is loaded, pass through zeros
        if self.ir_length == 0 {
            output.fill(0.0);
            return;
        }
        
        // Process in blocks
        for (in_sample, out_sample) in input.iter().zip(output.iter_mut()) {
            // Add input to buffer
            self.input_buffer[self.position] = Complex::new(*in_sample, 0.0);
            self.position += 1;
            
            // When buffer is full, process it
            if self.position >= self.block_size {
                self.process_block();
                self.position = 0;
            }
            
            // Output from overlap buffer
            *out_sample = self.overlap[0];
            
            // Shift overlap buffer
            let len = self.overlap.len();
            self.overlap.copy_within(1.., 0);
            self.overlap[len - 1] = 0.0;
        }
    }
    
    /// Resets the convolution processor state.
    ///
    /// This clears all internal buffers and resets the processing position.
    pub fn reset(&mut self) {
        self.input_buffer.fill(Complex::new(0.0, 0.0));
        self.output_buffer.fill(Complex::new(0.0, 0.0));
        self.overlap.fill(0.0);
        self.position = 0;
    }
    
    /// Returns the current sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
    
    /// Returns the current block size.
    pub fn block_size(&self) -> usize {
        self.block_size
    }
    
    /// Returns the length of the loaded impulse response in samples.
    pub fn impulse_response_length(&self) -> usize {
        self.ir_length
    }
    
    // Private helper methods
    
    fn resize_fft(&mut self, new_size: usize) {
        self.fft_size = new_size;
        
        let mut planner = FftPlanner::new();
        self.fft = planner.plan_fft_forward(new_size);
        self.ifft = planner.plan_fft_inverse(new_size);
        
        self.input_buffer.resize(new_size, Complex::new(0.0, 0.0));
        self.output_buffer.resize(new_size, Complex::new(0.0, 0.0));
        self.overlap.resize(new_size, 0.0);
        self.ir_fft.resize(new_size, Complex::new(0.0, 0.0));
    }
    
    fn process_block(&mut self) {
        // Zero-pad the rest of the input buffer
        for i in self.block_size..self.fft_size {
            self.input_buffer[i] = Complex::new(0.0, 0.0);
        }
        
        // FFT of input
        self.fft.process(&mut self.input_buffer);
        
        // Multiply with IR in frequency domain
        for i in 0..self.fft_size {
            self.output_buffer[i] = self.input_buffer[i] * self.ir_fft[i];
        }
        
        // If crossfading, also process with new IR
        if let Some(ref new_ir_fft) = self.new_ir_fft {
            let mut new_output = self.input_buffer.clone();
            for i in 0..self.fft_size {
                new_output[i] = self.input_buffer[i] * new_ir_fft[i];
            }
            
            // IFFT of new output
            self.ifft.process(&mut new_output);
            
            // IFFT of current output
            self.ifft.process(&mut self.output_buffer);
            
            // Crossfade between old and new
            let fade_length = self.crossfade_samples.min(self.block_size);
            for i in 0..self.fft_size {
                let old_sample = self.output_buffer[i].re / self.fft_size as f32;
                let new_sample = new_output[i].re / self.fft_size as f32;
                
                if self.crossfade_position < fade_length {
                    let fade = self.crossfade_position as f32 / fade_length as f32;
                    self.overlap[i] += old_sample * (1.0 - fade) + new_sample * fade;
                    self.crossfade_position += 1;
                } else {
                    self.overlap[i] += new_sample;
                }
            }
            
            // If crossfade is complete, switch to new IR
            if self.crossfade_position >= fade_length {
                self.ir_fft = new_ir_fft.clone();
                self.ir_length = self.new_ir_length;
                self.new_ir_fft = None;
                self.crossfade_position = 0;
            }
        } else {
            // IFFT
            self.ifft.process(&mut self.output_buffer);
            
            // Add to overlap buffer (normalize by FFT size)
            for i in 0..self.fft_size {
                self.overlap[i] += self.output_buffer[i].re / self.fft_size as f32;
            }
        }
    }
}

impl Default for Convolution {
    fn default() -> Self {
        Self::new(44100.0, 512)
    }
}

/// Stereo convolution processor.
///
/// This processor handles stereo convolution with separate impulse responses
/// for left and right channels, or a single mono impulse response applied to both.
pub struct StereoConvolution {
    left: Convolution,
    right: Convolution,
    is_stereo_ir: bool,
}

impl StereoConvolution {
    /// Creates a new stereo convolution processor.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate in Hz
    /// * `block_size` - The processing block size in samples
    pub fn new(sample_rate: f32, block_size: usize) -> Self {
        Self {
            left: Convolution::new(sample_rate, block_size),
            right: Convolution::new(sample_rate, block_size),
            is_stereo_ir: false,
        }
    }
    
    /// Loads a mono impulse response (applied to both channels).
    ///
    /// # Arguments
    ///
    /// * `ir` - The impulse response samples
    /// * `sample_rate` - The sample rate of the impulse response
    pub fn load_mono_impulse_response(&mut self, ir: &[f32], sample_rate: f32) -> Result<(), DspError> {
        self.left.load_impulse_response(ir, sample_rate)?;
        self.right.load_impulse_response(ir, sample_rate)?;
        self.is_stereo_ir = false;
        Ok(())
    }
    
    /// Loads a stereo impulse response (separate for each channel).
    ///
    /// # Arguments
    ///
    /// * `ir_left` - The left channel impulse response samples
    /// * `ir_right` - The right channel impulse response samples
    /// * `sample_rate` - The sample rate of the impulse response
    pub fn load_stereo_impulse_response(
        &mut self,
        ir_left: &[f32],
        ir_right: &[f32],
        sample_rate: f32,
    ) -> Result<(), DspError> {
        self.left.load_impulse_response(ir_left, sample_rate)?;
        self.right.load_impulse_response(ir_right, sample_rate)?;
        self.is_stereo_ir = true;
        Ok(())
    }
    
    /// Processes stereo audio through the convolution.
    ///
    /// # Arguments
    ///
    /// * `input_left` - Left channel input samples
    /// * `input_right` - Right channel input samples
    /// * `output_left` - Left channel output samples (will be overwritten)
    /// * `output_right` - Right channel output samples (will be overwritten)
    ///
    /// # Panics
    ///
    /// Panics if input and output slices have different lengths.
    pub fn process(
        &mut self,
        input_left: &[f32],
        input_right: &[f32],
        output_left: &mut [f32],
        output_right: &mut [f32],
    ) {
        assert_eq!(input_left.len(), input_right.len());
        assert_eq!(output_left.len(), output_right.len());
        assert_eq!(input_left.len(), output_left.len());
        
        self.left.process(input_left, output_left);
        self.right.process(input_right, output_right);
    }
    
    /// Resets both channels of the stereo convolution processor.
    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }
}

impl Default for StereoConvolution {
    fn default() -> Self {
        Self::new(44100.0, 512)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convolution_creation() {
        let conv = Convolution::new(44100.0, 512);
        assert_eq!(conv.sample_rate(), 44100.0);
        assert_eq!(conv.block_size(), 512);
        assert_eq!(conv.impulse_response_length(), 0);
    }
    
    #[test]
    fn test_load_impulse_response() {
        let mut conv = Convolution::new(44100.0, 512);
        let ir = vec![1.0, 0.5, 0.25, 0.125];
        
        let result = conv.load_impulse_response(&ir, 44100.0);
        assert!(result.is_ok());
        assert_eq!(conv.impulse_response_length(), 4);
    }
    
    #[test]
    fn test_sample_rate_validation() {
        let mut conv = Convolution::new(44100.0, 512);
        let ir = vec![1.0, 0.5, 0.25];
        
        // Wrong sample rate should fail
        let result = conv.load_impulse_response(&ir, 48000.0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_ir_rejected() {
        let mut conv = Convolution::new(44100.0, 512);
        let ir: Vec<f32> = vec![];
        
        let result = conv.load_impulse_response(&ir, 44100.0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_process_without_ir() {
        let mut conv = Convolution::new(44100.0, 512);
        let input = vec![1.0; 512];
        let mut output = vec![0.0; 512];
        
        conv.process(&input, &mut output);
        
        // Without IR, output should be zeros
        assert!(output.iter().all(|&x| x == 0.0));
    }
    
    #[test]
    fn test_process_with_impulse() {
        let mut conv = Convolution::new(44100.0, 512);
        let ir = vec![1.0]; // Unity impulse
        conv.load_impulse_response(&ir, 44100.0).unwrap();
        
        let input = vec![1.0; 512];
        let mut output = vec![0.0; 512];
        
        conv.process(&input, &mut output);
        
        // With unity impulse, output should eventually match input
        // (after the overlap-add delay)
    }
    
    #[test]
    fn test_reset() {
        let mut conv = Convolution::new(44100.0, 512);
        let ir = vec![1.0, 0.5];
        conv.load_impulse_response(&ir, 44100.0).unwrap();
        
        let input = vec![1.0; 512];
        let mut output = vec![0.0; 512];
        conv.process(&input, &mut output);
        
        conv.reset();
        
        // After reset, internal state should be cleared
        assert_eq!(conv.position, 0);
    }
    
    #[test]
    fn test_stereo_convolution() {
        let mut stereo = StereoConvolution::new(44100.0, 512);
        let ir = vec![1.0, 0.5, 0.25];
        
        stereo.load_mono_impulse_response(&ir, 44100.0).unwrap();
        
        let input_l = vec![1.0; 512];
        let input_r = vec![0.5; 512];
        let mut output_l = vec![0.0; 512];
        let mut output_r = vec![0.0; 512];
        
        stereo.process(&input_l, &input_r, &mut output_l, &mut output_r);
    }
}
