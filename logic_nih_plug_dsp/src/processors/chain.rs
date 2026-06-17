//! Processor chain for composing multiple audio processors.
//!
//! This module provides a trait for audio processors and a chain implementation
//! that allows multiple processors to be connected in sequence.

/// Trait for audio processors that can be chained together.
///
/// All processors must implement this trait to be used in a ProcessorChain.
/// The trait defines the lifecycle methods for preparing, processing, and
/// resetting processor state.
pub trait Processor: Send {
    /// Prepares the processor for audio processing.
    ///
    /// This should be called before processing audio, typically when
    /// the sample rate or buffer size changes.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz
    /// * `max_block_size` - Maximum block size that will be processed
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize);

    /// Processes a buffer of samples through the processor.
    ///
    /// # Arguments
    ///
    /// * `input` - Input buffer
    /// * `output` - Output buffer (must be same length as input)
    fn process(&mut self, input: &[f32], output: &mut [f32]);

    /// Resets the processor state.
    ///
    /// This should clear any internal state (e.g., filter history)
    /// without affecting parameter settings.
    fn reset(&mut self);
}

/// A chain of audio processors connected in sequence.
///
/// ProcessorChain allows multiple processors to be connected together,
/// with the output of each processor feeding into the input of the next.
/// This is useful for creating complex audio processing pipelines.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::processors::chain::{Processor, ProcessorChain};
/// use nih_plug_dsp::processors::gain::Gain;
/// use nih_plug_dsp::processors::bias::Bias;
///
/// let mut chain = ProcessorChain::new();
///
/// let mut gain = Gain::new();
/// gain.set_gain_db(6.0);
/// chain.add(gain);
///
/// let mut bias = Bias::new();
/// bias.set_bias(0.1);
/// chain.add(bias);
///
/// chain.prepare(44100.0, 512);
///
/// let input = vec![0.5; 512];
/// let mut output = vec![0.0; 512];
/// chain.process(&input, &mut output);
/// ```
pub struct ProcessorChain {
    processors: Vec<Box<dyn Processor>>,
    temp_buffer: Vec<f32>,
}

impl ProcessorChain {
    /// Creates a new empty processor chain.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::chain::ProcessorChain;
    ///
    /// let chain = ProcessorChain::new();
    /// ```
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            temp_buffer: Vec::new(),
        }
    }

    /// Adds a processor to the end of the chain.
    ///
    /// # Arguments
    ///
    /// * `processor` - The processor to add
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::chain::ProcessorChain;
    /// use nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut chain = ProcessorChain::new();
    /// let gain = Gain::new();
    /// chain.add(gain);
    /// ```
    pub fn add<P: Processor + 'static>(&mut self, processor: P) {
        self.processors.push(Box::new(processor));
    }

    /// Gets a reference to a processor in the chain by index.
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the processor to retrieve
    ///
    /// # Returns
    ///
    /// Some reference to the processor if index is valid, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::chain::ProcessorChain;
    /// use nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut chain = ProcessorChain::new();
    /// chain.add(Gain::new());
    ///
    /// let processor = chain.get(0);
    /// assert!(processor.is_some());
    /// ```
    pub fn get(&self, index: usize) -> Option<&dyn Processor> {
        self.processors.get(index).map(|p| p.as_ref())
    }

    /// Gets a mutable reference to a processor in the chain by index.
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the processor to retrieve
    ///
    /// # Returns
    ///
    /// Some mutable reference to the processor if index is valid, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::chain::ProcessorChain;
    /// use nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut chain = ProcessorChain::new();
    /// chain.add(Gain::new());
    ///
    /// let processor = chain.get_mut(0);
    /// assert!(processor.is_some());
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut (dyn Processor + '_)> {
        match self.processors.get_mut(index) {
            Some(p) => Some(p.as_mut()),
            None => None,
        }
    }

    /// Returns the number of processors in the chain.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::chain::ProcessorChain;
    /// use nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut chain = ProcessorChain::new();
    /// assert_eq!(chain.len(), 0);
    ///
    /// chain.add(Gain::new());
    /// assert_eq!(chain.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.processors.len()
    }

    /// Returns true if the chain contains no processors.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::chain::ProcessorChain;
    ///
    /// let chain = ProcessorChain::new();
    /// assert!(chain.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }
}

impl Processor for ProcessorChain {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        // Prepare all processors in the chain
        for processor in &mut self.processors {
            processor.prepare(sample_rate, max_block_size);
        }

        // Allocate temp buffer for intermediate processing
        self.temp_buffer.resize(max_block_size, 0.0);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        if self.processors.is_empty() {
            // No processors, just copy input to output
            output.copy_from_slice(input);
            return;
        }

        let len = input.len();

        // Process through the chain
        if self.processors.len() == 1 {
            // Single processor, process directly
            self.processors[0].process(input, output);
        } else {
            // Multiple processors, use temp buffer for intermediate results
            // First processor: input -> temp_buffer
            self.processors[0].process(input, &mut self.temp_buffer[..len]);

            // Middle processors: temp_buffer -> output -> temp_buffer
            for i in 1..self.processors.len() - 1 {
                output[..len].copy_from_slice(&self.temp_buffer[..len]);
                self.processors[i].process(&output[..len], &mut self.temp_buffer[..len]);
            }

            // Last processor: temp_buffer -> output
            let last_idx = self.processors.len() - 1;
            self.processors[last_idx].process(&self.temp_buffer[..len], output);
        }
    }

    fn reset(&mut self) {
        // Reset all processors in the chain
        for processor in &mut self.processors {
            processor.reset();
        }
    }
}

impl Default for ProcessorChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test processor that multiplies by a factor
    struct MultiplyProcessor {
        factor: f32,
    }

    impl MultiplyProcessor {
        fn new(factor: f32) -> Self {
            Self { factor }
        }
    }

    impl Processor for MultiplyProcessor {
        fn prepare(&mut self, _sample_rate: f32, _max_block_size: usize) {}

        fn process(&mut self, input: &[f32], output: &mut [f32]) {
            for (inp, out) in input.iter().zip(output.iter_mut()) {
                *out = inp * self.factor;
            }
        }

        fn reset(&mut self) {}
    }

    // Simple test processor that adds a value
    struct AddProcessor {
        value: f32,
    }

    impl AddProcessor {
        fn new(value: f32) -> Self {
            Self { value }
        }
    }

    impl Processor for AddProcessor {
        fn prepare(&mut self, _sample_rate: f32, _max_block_size: usize) {}

        fn process(&mut self, input: &[f32], output: &mut [f32]) {
            for (inp, out) in input.iter().zip(output.iter_mut()) {
                *out = inp + self.value;
            }
        }

        fn reset(&mut self) {}
    }

    #[test]
    fn test_empty_chain() {
        let mut chain = ProcessorChain::new();
        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());

        chain.prepare(44100.0, 512);

        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        chain.process(&input, &mut output);

        // Empty chain should pass through
        assert_eq!(output, input);
    }

    #[test]
    fn test_single_processor() {
        let mut chain = ProcessorChain::new();
        chain.add(MultiplyProcessor::new(2.0));

        chain.prepare(44100.0, 512);

        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        chain.process(&input, &mut output);

        assert_eq!(output, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_multiple_processors() {
        let mut chain = ProcessorChain::new();
        chain.add(MultiplyProcessor::new(2.0));
        chain.add(AddProcessor::new(1.0));

        chain.prepare(44100.0, 512);

        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        chain.process(&input, &mut output);

        // (input * 2) + 1
        assert_eq!(output, vec![3.0, 5.0, 7.0]);
    }

    #[test]
    fn test_chain_composition() {
        let mut chain = ProcessorChain::new();
        chain.add(AddProcessor::new(1.0));
        chain.add(MultiplyProcessor::new(2.0));
        chain.add(AddProcessor::new(3.0));

        chain.prepare(44100.0, 512);

        let input = vec![1.0];
        let mut output = vec![0.0];
        chain.process(&input, &mut output);

        // ((1 + 1) * 2) + 3 = 7
        assert_eq!(output[0], 7.0);
    }

    #[test]
    fn test_get_processor() {
        let mut chain = ProcessorChain::new();
        chain.add(MultiplyProcessor::new(2.0));
        chain.add(AddProcessor::new(1.0));

        assert!(chain.get(0).is_some());
        assert!(chain.get(1).is_some());
        assert!(chain.get(2).is_none());
    }

    #[test]
    fn test_get_mut_processor() {
        let mut chain = ProcessorChain::new();
        chain.add(MultiplyProcessor::new(2.0));

        assert!(chain.get_mut(0).is_some());
        assert!(chain.get_mut(1).is_none());
    }

    #[test]
    fn test_chain_len() {
        let mut chain = ProcessorChain::new();
        assert_eq!(chain.len(), 0);

        chain.add(MultiplyProcessor::new(2.0));
        assert_eq!(chain.len(), 1);

        chain.add(AddProcessor::new(1.0));
        assert_eq!(chain.len(), 2);
    }
}
