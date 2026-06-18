//! Circular waveform capture buffer for oscilloscope-style display.
//!
//! An [`Oscilloscope`] captures audio samples into a fixed-size circular
//! buffer, recording both **min** and **max** sample values per
//! "block". This is the standard technique for waveform visualisation:
//! the renderer draws min–max bars from the buffer, producing a faithful
//! preview of the waveform shape without storing every individual sample.
//!
//! # Quick start
//!
//! ```
//! use logic_nih_plug_dsp::analysis::oscilloscope::Oscilloscope;
//!
//! let mut osc = Oscilloscope::new(1024);
//!
//! // Push a block of samples:
//! let block = vec![0.3f32, -0.2, 0.5, -0.1, 0.0];
//! osc.push_block(&block);
//!
//! assert_eq!(osc.len(), 1);
//!
//! // Query the min/max for the most recent block:
//! let b = osc.last_block().unwrap();
//! assert!((b.min - (-0.2)).abs() < 0.01);
//! assert!((b.max - 0.5).abs() < 0.01);
//! ```

/// A single captured block — the minimum and maximum sample values
/// within that block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    /// Minimum sample value in this block.
    pub min: f32,
    /// Maximum sample value in this block.
    pub max: f32,
}

impl Block {
    /// Creates a new block with the given min/max values.
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    /// Returns the midpoint (average) of min and max.
    pub fn mid(&self) -> f32 {
        (self.min + self.max) * 0.5
    }

    /// Returns the peak-to-peak amplitude.
    pub fn peak_to_peak(&self) -> f32 {
        self.max - self.min
    }
}

/// A circular buffer that captures min/max blocks for waveform display.
///
/// The capacity is set at construction time and represents the number
/// of *blocks* (not individual samples). Each call to
/// [`push_block`](Self::push_block) computes the min/max of the
/// provided samples and stores one [`Block`] entry.
///
/// When the buffer is full the oldest entry is silently overwritten,
/// giving the standard "scrolling oscilloscope" behaviour.
#[derive(Debug, Clone)]
pub struct Oscilloscope {
    /// Ring buffer of blocks.
    buffer: Vec<Block>,
    /// Write position (next slot to write).
    write_pos: usize,
    /// Number of valid entries (capped at capacity).
    count: usize,
    /// Total capacity in blocks.
    capacity: usize,
}

impl Oscilloscope {
    /// Creates a new oscilloscope buffer that can hold `capacity` blocks.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be > 0");
        Self {
            buffer: vec![Block::new(0.0, 0.0); capacity],
            write_pos: 0,
            count: 0,
            capacity,
        }
    }

    /// Returns the capacity (maximum number of blocks).
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of blocks currently stored.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if no blocks have been captured yet.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Pushes a block of samples into the buffer.
    ///
    /// The min and max of `samples` are computed and stored as a
    /// single [`Block`] entry. If the buffer is full, the oldest
    /// entry is overwritten.
    pub fn push_block(&mut self, samples: &[f32]) {
        let block = if samples.is_empty() {
            Block::new(0.0, 0.0)
        } else {
            let mut min = samples[0];
            let mut max = samples[0];
            for &s in &samples[1..] {
                if s < min {
                    min = s;
                }
                if s > max {
                    max = s;
                }
            }
            Block::new(min, max)
        };

        self.buffer[self.write_pos] = block;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Returns the most recently pushed block, or `None` if empty.
    pub fn last_block(&self) -> Option<Block> {
        if self.count == 0 {
            None
        } else {
            let idx = if self.write_pos == 0 {
                self.capacity - 1
            } else {
                self.write_pos - 1
            };
            Some(self.buffer[idx])
        }
    }

    /// Returns a block by its *age* index (0 = most recent, 1 = second
    /// most recent, etc.), or `None` if the index is out of range.
    pub fn block_by_age(&self, age: usize) -> Option<Block> {
        if age >= self.count {
            return None;
        }
        let actual = if self.write_pos == 0 {
            self.capacity - 1 - age
        } else {
            // write_pos points to the next write slot, so the most
            // recent entry is at write_pos - 1 (mod capacity).
            (self.write_pos + self.capacity - 1 - age) % self.capacity
        };
        Some(self.buffer[actual])
    }

    /// Returns an iterator over all stored blocks in chronological order
    /// (oldest first).
    pub fn iter(&self) -> OscilloscopeIter<'_> {
        OscilloscopeIter {
            scope: self,
            index: 0,
        }
    }

    /// Clears all captured blocks.
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.count = 0;
    }
}

/// Iterator over [`Oscilloscope`] blocks in chronological order.
pub struct OscilloscopeIter<'a> {
    scope: &'a Oscilloscope,
    index: usize,
}

impl<'a> Iterator for OscilloscopeIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.scope.count {
            return None;
        }

        // Chronological order: oldest first.
        let read_pos = if self.scope.count < self.scope.capacity {
            // Buffer hasn't wrapped yet: oldest is at index 0.
            self.index
        } else {
            // Buffer has wrapped: oldest is at write_pos.
            (self.scope.write_pos + self.index) % self.scope.capacity
        };

        self.index += 1;
        Some(self.scope.buffer[read_pos])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.scope.count.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for OscilloscopeIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let osc = Oscilloscope::new(128);
        assert!(osc.is_empty());
        assert_eq!(osc.len(), 0);
        assert!(osc.last_block().is_none());
    }

    #[test]
    fn test_push_single_block() {
        let mut osc = Oscilloscope::new(128);
        osc.push_block(&[0.5, -0.3, 0.8, -0.1]);
        assert_eq!(osc.len(), 1);

        let block = osc.last_block().unwrap();
        assert!((block.min - (-0.3)).abs() < 1e-6);
        assert!((block.max - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_circular_overwrite() {
        let mut osc = Oscilloscope::new(3);

        osc.push_block(&[1.0]);
        osc.push_block(&[2.0]);
        osc.push_block(&[3.0]);
        osc.push_block(&[4.0]); // overwrites [1.0]

        assert_eq!(osc.len(), 3);
        // Most recent is [4.0]
        assert_eq!(osc.last_block().unwrap().max, 4.0);
        // Oldest should be [2.0]
        assert_eq!(osc.block_by_age(2).unwrap().max, 2.0);
    }

    #[test]
    fn test_block_mid_and_pp() {
        let b = Block::new(-0.5, 1.5);
        assert!((b.mid() - 0.5).abs() < 1e-6);
        assert!((b.peak_to_peak() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_iter_chronological() {
        let mut osc = Oscilloscope::new(4);
        osc.push_block(&[10.0]);
        osc.push_block(&[20.0]);
        osc.push_block(&[30.0]);

        let values: Vec<f32> = osc.iter().map(|b| b.max).collect();
        assert_eq!(values, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_iter_after_wrap() {
        let mut osc = Oscilloscope::new(3);
        osc.push_block(&[10.0]);
        osc.push_block(&[20.0]);
        osc.push_block(&[30.0]);
        osc.push_block(&[40.0]); // wraps, oldest = 20

        let values: Vec<f32> = osc.iter().map(|b| b.max).collect();
        assert_eq!(values, vec![20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_empty_block() {
        let mut osc = Oscilloscope::new(4);
        osc.push_block(&[]);
        assert_eq!(osc.len(), 1);
        let b = osc.last_block().unwrap();
        assert_eq!(b.min, 0.0);
        assert_eq!(b.max, 0.0);
    }

    #[test]
    fn test_single_sample_block() {
        let mut osc = Oscilloscope::new(4);
        osc.push_block(&[-0.5]);
        let b = osc.last_block().unwrap();
        assert_eq!(b.min, -0.5);
        assert_eq!(b.max, -0.5);
    }

    #[test]
    fn test_clear() {
        let mut osc = Oscilloscope::new(4);
        osc.push_block(&[1.0]);
        osc.push_block(&[2.0]);
        osc.clear();
        assert!(osc.is_empty());
        assert!(osc.last_block().is_none());
    }

    #[test]
    fn test_block_by_age_out_of_range() {
        let mut osc = Oscilloscope::new(4);
        osc.push_block(&[1.0]);
        assert!(osc.block_by_age(0).is_some());
        assert!(osc.block_by_age(1).is_none());
    }
}
