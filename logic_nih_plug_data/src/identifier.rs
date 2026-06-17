//! Strongly-typed identifiers for `ValueTree` properties, child nodes and types.
//!
//! [`Identifier`] wraps an interned-like [`Arc<str>`] so that property names and node
//! types can be compared and hashed cheaply without copying the underlying string.
//! This mirrors JUCE's `juce::Identifier` class.

use std::borrow::Borrow;
use std::fmt;
use std::sync::Arc;

/// A strongly-typed name used for `ValueTree` properties, child nodes and types.
///
/// Two identifiers are equal if their underlying strings are equal. Cloning is cheap
/// because the string is reference-counted.
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier(Arc<str>);

impl Identifier {
    /// Creates a new identifier from anything that can be converted into an
    /// `Arc<str>`.
    pub fn new(s: impl Into<Arc<str>>) -> Self {
        Self(s.into())
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` if this identifier was constructed from an empty string.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the length of the underlying string in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier({:?})", &self.0)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<&Identifier> for Identifier {
    fn from(id: &Identifier) -> Self {
        id.clone()
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<&String> for Identifier {
    fn from(s: &String) -> Self {
        Self(Arc::from(s.as_str()))
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_and_hashing() {
        let a = Identifier::new("gain");
        let b = Identifier::new("gain");
        let c = Identifier::new("freq");
        assert_eq!(a, b);
        assert_ne!(a, c);

        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
    }

    #[test]
    fn display_and_str_round_trip() {
        let id = Identifier::new("cutoff");
        assert_eq!(id.as_str(), "cutoff");
        assert_eq!(format!("{id}"), "cutoff");
    }

    #[test]
    fn from_string_and_str() {
        let from_str: Identifier = "name".into();
        let from_string: Identifier = String::from("name").into();
        let from_str_ref: Identifier = (&String::from("name")).into();
        assert_eq!(from_str, from_string);
        assert_eq!(from_str, from_str_ref);
    }

    #[test]
    fn empty_identifier() {
        let id = Identifier::new("");
        assert!(id.is_empty());
        assert_eq!(id.len(), 0);
    }
}
