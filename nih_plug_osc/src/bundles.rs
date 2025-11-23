//! OSC bundle implementation.
//!
//! Provides utilities for working with OSC bundles - timestamped groups of messages.
//!
//! # Examples
//!
//! ## Creating a Bundle
//!
//! ```
//! use nih_plug_osc::{OscBundle, OscMessage, OscType, OscTime};
//!
//! let mut bundle = OscBundle::immediate();
//! bundle.add_message(OscMessage::new("/synth/note", vec![OscType::Int(60)]));
//! bundle.add_message(OscMessage::new("/synth/velocity", vec![OscType::Float(0.8)]));
//! ```
//!
//! ## Nested Bundles
//!
//! ```
//! use nih_plug_osc::{OscBundle, OscMessage, OscType, OscTime};
//!
//! let mut inner = OscBundle::immediate();
//! inner.add_message(OscMessage::new("/test1", vec![OscType::Int(1)]));
//!
//! let mut outer = OscBundle::immediate();
//! outer.add_bundle(inner);
//! outer.add_message(OscMessage::new("/test2", vec![OscType::Int(2)]));
//! ```
//!
//! ## Scheduled Bundles
//!
//! ```
//! use nih_plug_osc::{OscBundle, OscMessage, OscType, OscTime};
//!
//! // Schedule for a specific time
//! let time = OscTime::new(3600, 0);
//! let mut bundle = OscBundle::new(time);
//! bundle.add_message(OscMessage::new("/trigger", vec![]));
//! ```

use crate::message::{OscBundle, OscMessage, OscPacket, OscTime};

/// Builder for creating OSC bundles with a fluent API.
///
/// # Examples
///
/// ```
/// use nih_plug_osc::{OscMessage, OscType, OscTime};
/// use nih_plug_osc::bundles::BundleBuilder;
///
/// let bundle = BundleBuilder::new()
///     .with_time_tag(OscTime::immediate())
///     .add_message(OscMessage::new("/test1", vec![OscType::Int(1)]))
///     .add_message(OscMessage::new("/test2", vec![OscType::Int(2)]))
///     .build();
///
/// assert_eq!(bundle.packets.len(), 2);
/// ```
pub struct BundleBuilder {
    bundle: OscBundle,
}

impl BundleBuilder {
    /// Creates a new bundle builder with immediate time tag.
    pub fn new() -> Self {
        Self {
            bundle: OscBundle::immediate(),
        }
    }

    /// Sets the time tag for the bundle.
    pub fn with_time_tag(mut self, time_tag: OscTime) -> Self {
        self.bundle.time_tag = time_tag;
        self
    }

    /// Adds a message to the bundle.
    pub fn add_message(mut self, message: OscMessage) -> Self {
        self.bundle.add_message(message);
        self
    }

    /// Adds a nested bundle to the bundle.
    pub fn add_bundle(mut self, bundle: OscBundle) -> Self {
        self.bundle.add_bundle(bundle);
        self
    }

    /// Adds a packet to the bundle.
    pub fn add_packet(mut self, packet: OscPacket) -> Self {
        self.bundle.add_packet(packet);
        self
    }

    /// Builds the final bundle.
    pub fn build(self) -> OscBundle {
        self.bundle
    }
}

impl Default for BundleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilities for working with OSC bundles.
pub struct BundleUtils;

impl BundleUtils {
    /// Flattens a bundle into a list of messages, recursively extracting
    /// messages from nested bundles.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscBundle, OscMessage, OscType};
    /// use nih_plug_osc::bundles::BundleUtils;
    ///
    /// let mut inner = OscBundle::immediate();
    /// inner.add_message(OscMessage::new("/inner", vec![OscType::Int(1)]));
    ///
    /// let mut outer = OscBundle::immediate();
    /// outer.add_bundle(inner);
    /// outer.add_message(OscMessage::new("/outer", vec![OscType::Int(2)]));
    ///
    /// let messages = BundleUtils::flatten(&outer);
    /// assert_eq!(messages.len(), 2);
    /// ```
    pub fn flatten(bundle: &OscBundle) -> Vec<OscMessage> {
        let mut messages = Vec::new();
        Self::flatten_recursive(&bundle.packets, &mut messages);
        messages
    }

    fn flatten_recursive(packets: &[OscPacket], messages: &mut Vec<OscMessage>) {
        for packet in packets {
            match packet {
                OscPacket::Message(msg) => messages.push(msg.clone()),
                OscPacket::Bundle(bundle) => Self::flatten_recursive(&bundle.packets, messages),
            }
        }
    }

    /// Counts the total number of messages in a bundle, including nested bundles.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscBundle, OscMessage, OscType};
    /// use nih_plug_osc::bundles::BundleUtils;
    ///
    /// let mut bundle = OscBundle::immediate();
    /// bundle.add_message(OscMessage::new("/test1", vec![]));
    /// bundle.add_message(OscMessage::new("/test2", vec![]));
    ///
    /// assert_eq!(BundleUtils::count_messages(&bundle), 2);
    /// ```
    pub fn count_messages(bundle: &OscBundle) -> usize {
        Self::count_messages_recursive(&bundle.packets)
    }

    fn count_messages_recursive(packets: &[OscPacket]) -> usize {
        packets
            .iter()
            .map(|packet| match packet {
                OscPacket::Message(_) => 1,
                OscPacket::Bundle(bundle) => Self::count_messages_recursive(&bundle.packets),
            })
            .sum()
    }

    /// Gets the maximum nesting depth of bundles.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscBundle, OscMessage, OscType};
    /// use nih_plug_osc::bundles::BundleUtils;
    ///
    /// let mut inner = OscBundle::immediate();
    /// inner.add_message(OscMessage::new("/test", vec![]));
    ///
    /// let mut outer = OscBundle::immediate();
    /// outer.add_bundle(inner);
    ///
    /// assert_eq!(BundleUtils::depth(&outer), 2);
    /// ```
    pub fn depth(bundle: &OscBundle) -> usize {
        1 + Self::depth_recursive(&bundle.packets)
    }

    fn depth_recursive(packets: &[OscPacket]) -> usize {
        packets
            .iter()
            .map(|packet| match packet {
                OscPacket::Message(_) => 0,
                OscPacket::Bundle(bundle) => 1 + Self::depth_recursive(&bundle.packets),
            })
            .max()
            .unwrap_or(0)
    }

    /// Filters messages in a bundle by address pattern.
    ///
    /// Returns a new bundle containing only messages whose addresses match the pattern.
    /// Supports simple wildcard matching with '*'.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscBundle, OscMessage, OscType};
    /// use nih_plug_osc::bundles::BundleUtils;
    ///
    /// let mut bundle = OscBundle::immediate();
    /// bundle.add_message(OscMessage::new("/synth/note", vec![OscType::Int(60)]));
    /// bundle.add_message(OscMessage::new("/synth/velocity", vec![OscType::Float(0.8)]));
    /// bundle.add_message(OscMessage::new("/effect/reverb", vec![OscType::Float(0.5)]));
    ///
    /// let filtered = BundleUtils::filter_by_address(&bundle, "/synth/*");
    /// assert_eq!(BundleUtils::count_messages(&filtered), 2);
    /// ```
    pub fn filter_by_address(bundle: &OscBundle, pattern: &str) -> OscBundle {
        let mut result = OscBundle::new(bundle.time_tag);
        Self::filter_recursive(&bundle.packets, pattern, &mut result.packets);
        result
    }

    fn filter_recursive(packets: &[OscPacket], pattern: &str, result: &mut Vec<OscPacket>) {
        for packet in packets {
            match packet {
                OscPacket::Message(msg) => {
                    if Self::matches_pattern(&msg.address, pattern) {
                        result.push(OscPacket::Message(msg.clone()));
                    }
                }
                OscPacket::Bundle(bundle) => {
                    let mut filtered_bundle = OscBundle::new(bundle.time_tag);
                    Self::filter_recursive(&bundle.packets, pattern, &mut filtered_bundle.packets);
                    if !filtered_bundle.packets.is_empty() {
                        result.push(OscPacket::Bundle(filtered_bundle));
                    }
                }
            }
        }
    }

    fn matches_pattern(address: &str, pattern: &str) -> bool {
        // Simple wildcard matching
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return address.starts_with(prefix);
        }

        address == pattern
    }

    /// Merges multiple bundles into a single bundle.
    ///
    /// The resulting bundle uses the time tag from the first bundle.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscBundle, OscMessage, OscType};
    /// use nih_plug_osc::bundles::BundleUtils;
    ///
    /// let mut bundle1 = OscBundle::immediate();
    /// bundle1.add_message(OscMessage::new("/test1", vec![]));
    ///
    /// let mut bundle2 = OscBundle::immediate();
    /// bundle2.add_message(OscMessage::new("/test2", vec![]));
    ///
    /// let merged = BundleUtils::merge(&[bundle1, bundle2]);
    /// assert_eq!(BundleUtils::count_messages(&merged), 2);
    /// ```
    pub fn merge(bundles: &[OscBundle]) -> OscBundle {
        if bundles.is_empty() {
            return OscBundle::immediate();
        }

        let mut result = OscBundle::new(bundles[0].time_tag);
        for bundle in bundles {
            for packet in &bundle.packets {
                result.add_packet(packet.clone());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::OscType;

    #[test]
    fn test_bundle_builder() {
        let bundle = BundleBuilder::new()
            .add_message(OscMessage::new("/test1", vec![OscType::Int(1)]))
            .add_message(OscMessage::new("/test2", vec![OscType::Int(2)]))
            .build();

        assert_eq!(bundle.packets.len(), 2);
        assert!(bundle.time_tag.is_immediate());
    }

    #[test]
    fn test_bundle_builder_with_time_tag() {
        let time = OscTime::new(100, 200);
        let bundle = BundleBuilder::new().with_time_tag(time).build();

        assert_eq!(bundle.time_tag.seconds, 100);
        assert_eq!(bundle.time_tag.fractional, 200);
    }

    #[test]
    fn test_bundle_builder_nested() {
        let inner = BundleBuilder::new()
            .add_message(OscMessage::new("/inner", vec![]))
            .build();

        let outer = BundleBuilder::new().add_bundle(inner).build();

        assert_eq!(outer.packets.len(), 1);
        match &outer.packets[0] {
            OscPacket::Bundle(_) => {}
            _ => panic!("Expected bundle"),
        }
    }

    #[test]
    fn test_flatten_simple() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/test1", vec![]));
        bundle.add_message(OscMessage::new("/test2", vec![]));

        let messages = BundleUtils::flatten(&bundle);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].address, "/test1");
        assert_eq!(messages[1].address, "/test2");
    }

    #[test]
    fn test_flatten_nested() {
        let mut inner = OscBundle::immediate();
        inner.add_message(OscMessage::new("/inner", vec![]));

        let mut outer = OscBundle::immediate();
        outer.add_bundle(inner);
        outer.add_message(OscMessage::new("/outer", vec![]));

        let messages = BundleUtils::flatten(&outer);
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_count_messages() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/test1", vec![]));
        bundle.add_message(OscMessage::new("/test2", vec![]));

        assert_eq!(BundleUtils::count_messages(&bundle), 2);
    }

    #[test]
    fn test_count_messages_nested() {
        let mut inner = OscBundle::immediate();
        inner.add_message(OscMessage::new("/inner1", vec![]));
        inner.add_message(OscMessage::new("/inner2", vec![]));

        let mut outer = OscBundle::immediate();
        outer.add_bundle(inner);
        outer.add_message(OscMessage::new("/outer", vec![]));

        assert_eq!(BundleUtils::count_messages(&outer), 3);
    }

    #[test]
    fn test_depth_simple() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/test", vec![]));

        assert_eq!(BundleUtils::depth(&bundle), 1);
    }

    #[test]
    fn test_depth_nested() {
        let mut inner = OscBundle::immediate();
        inner.add_message(OscMessage::new("/inner", vec![]));

        let mut outer = OscBundle::immediate();
        outer.add_bundle(inner);

        assert_eq!(BundleUtils::depth(&outer), 2);
    }

    #[test]
    fn test_depth_deeply_nested() {
        let mut level3 = OscBundle::immediate();
        level3.add_message(OscMessage::new("/level3", vec![]));

        let mut level2 = OscBundle::immediate();
        level2.add_bundle(level3);

        let mut level1 = OscBundle::immediate();
        level1.add_bundle(level2);

        assert_eq!(BundleUtils::depth(&level1), 3);
    }

    #[test]
    fn test_filter_by_address_exact() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/test1", vec![]));
        bundle.add_message(OscMessage::new("/test2", vec![]));

        let filtered = BundleUtils::filter_by_address(&bundle, "/test1");
        assert_eq!(BundleUtils::count_messages(&filtered), 1);

        let messages = BundleUtils::flatten(&filtered);
        assert_eq!(messages[0].address, "/test1");
    }

    #[test]
    fn test_filter_by_address_wildcard() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/synth/note", vec![]));
        bundle.add_message(OscMessage::new("/synth/velocity", vec![]));
        bundle.add_message(OscMessage::new("/effect/reverb", vec![]));

        let filtered = BundleUtils::filter_by_address(&bundle, "/synth/*");
        assert_eq!(BundleUtils::count_messages(&filtered), 2);
    }

    #[test]
    fn test_filter_by_address_all() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/test1", vec![]));
        bundle.add_message(OscMessage::new("/test2", vec![]));

        let filtered = BundleUtils::filter_by_address(&bundle, "*");
        assert_eq!(BundleUtils::count_messages(&filtered), 2);
    }

    #[test]
    fn test_filter_nested_bundles() {
        let mut inner = OscBundle::immediate();
        inner.add_message(OscMessage::new("/synth/note", vec![]));
        inner.add_message(OscMessage::new("/effect/reverb", vec![]));

        let mut outer = OscBundle::immediate();
        outer.add_bundle(inner);
        outer.add_message(OscMessage::new("/synth/velocity", vec![]));

        let filtered = BundleUtils::filter_by_address(&outer, "/synth/*");
        assert_eq!(BundleUtils::count_messages(&filtered), 2);
    }

    #[test]
    fn test_merge_empty() {
        let merged = BundleUtils::merge(&[]);
        assert!(merged.time_tag.is_immediate());
        assert_eq!(merged.packets.len(), 0);
    }

    #[test]
    fn test_merge_single() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/test", vec![]));

        let merged = BundleUtils::merge(&[bundle]);
        assert_eq!(BundleUtils::count_messages(&merged), 1);
    }

    #[test]
    fn test_merge_multiple() {
        let mut bundle1 = OscBundle::immediate();
        bundle1.add_message(OscMessage::new("/test1", vec![]));

        let mut bundle2 = OscBundle::immediate();
        bundle2.add_message(OscMessage::new("/test2", vec![]));

        let mut bundle3 = OscBundle::immediate();
        bundle3.add_message(OscMessage::new("/test3", vec![]));

        let merged = BundleUtils::merge(&[bundle1, bundle2, bundle3]);
        assert_eq!(BundleUtils::count_messages(&merged), 3);
    }

    #[test]
    fn test_merge_preserves_time_tag() {
        let time = OscTime::new(100, 200);
        let mut bundle1 = OscBundle::new(time);
        bundle1.add_message(OscMessage::new("/test1", vec![]));

        let mut bundle2 = OscBundle::immediate();
        bundle2.add_message(OscMessage::new("/test2", vec![]));

        let merged = BundleUtils::merge(&[bundle1, bundle2]);
        assert_eq!(merged.time_tag.seconds, 100);
        assert_eq!(merged.time_tag.fractional, 200);
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(BundleUtils::matches_pattern("/test", "/test"));
        assert!(!BundleUtils::matches_pattern("/test", "/other"));
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(BundleUtils::matches_pattern("/synth/note", "/synth/*"));
        assert!(BundleUtils::matches_pattern("/synth/velocity", "/synth/*"));
        assert!(!BundleUtils::matches_pattern("/effect/reverb", "/synth/*"));
    }

    #[test]
    fn test_matches_pattern_all() {
        assert!(BundleUtils::matches_pattern("/anything", "*"));
        assert!(BundleUtils::matches_pattern("/test/nested/deep", "*"));
    }
}
