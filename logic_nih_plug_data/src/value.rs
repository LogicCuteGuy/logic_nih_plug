//! [`Value`] — a `var`-style dynamic value used by [`crate::ValueTree`].
//!
//! JUCE's `var` supports a wide range of runtime types. We model the same set with an
//! enum that mirrors JUCE's most common variants plus binary blobs for arbitrary
//! payloads. The `ValueWithDefault` helper is a [`Value`] that falls back to a
//! user-supplied default when the underlying source is [`Value::Null`] or missing.

use std::fmt;

/// A dynamic value that can be stored on a `ValueTree`.
///
/// `Value` is the Rust equivalent of JUCE's `var` — a tagged union of the common
/// scalar and collection types that get serialized in plugin state and presets.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Value {
    /// No value / null.
    #[default]
    Null,
    /// Boolean.
    Bool(bool),
    /// 64-bit signed integer. JUCE uses 64-bit `var` integers; we follow that.
    Int(i64),
    /// 64-bit float. Stored as `f64` so it can round-trip through JSON / XML losslessly.
    Double(f64),
    /// UTF-8 string.
    String(String),
    /// Heterogeneous list of [`Value`]s.
    Array(Vec<Value>),
    /// Arbitrary binary blob.
    Binary(Vec<u8>),
}

impl Value {
    /// Returns `true` if this value is [`Value::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns the value as `i64` if it is [`Value::Int`] or coercible from
    /// [`Value::Double`] (truncating toward zero). Returns `fallback` otherwise.
    pub fn as_int_or(&self, fallback: i64) -> i64 {
        match self {
            Value::Int(n) => *n,
            Value::Double(f) => *f as i64,
            _ => fallback,
        }
    }

    /// Returns the value as `f64` if it is [`Value::Double`] or coercible from
    /// [`Value::Int`]. Returns `fallback` otherwise.
    pub fn as_double_or(&self, fallback: f64) -> f64 {
        match self {
            Value::Double(f) => *f,
            Value::Int(n) => *n as f64,
            _ => fallback,
        }
    }

    /// Returns the value as `bool`. Mirrors JUCE's loose bool coercion: numbers are
    /// truthy if non-zero; strings are truthy if non-empty.
    pub fn as_bool_or(&self, fallback: bool) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Double(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Null => fallback,
            _ => true,
        }
    }

    /// Returns the value as a borrowed `&str` if it is [`Value::String`].
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Double(x) => write!(f, "{x}"),
            Value::String(s) => write!(f, "{s:?}"),
            Value::Array(items) => {
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str("]")
            }
            Value::Binary(bytes) => {
                f.write_str("Binary(")?;
                write!(f, "{} bytes", bytes.len())?;
                f.write_str(")")
            }
        }
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v as i64)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Int(v as i64)
    }
}

impl From<usize> for Value {
    fn from(v: usize) -> Self {
        Value::Int(v as i64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Double(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Double(v as f64)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Binary(v)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Array(v.into_iter().map(Into::into).collect())
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        assert_eq!(Value::from(42i64), Value::Int(42));
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from("hi"), Value::String("hi".to_owned()));
        assert_eq!(Value::from(3.14_f64), Value::Double(3.14));
    }

    #[test]
    fn as_int_or_coerces_doubles() {
        assert_eq!(Value::Int(5).as_int_or(0), 5);
        assert_eq!(Value::Double(3.7).as_int_or(0), 3);
        assert_eq!(Value::String("nope".into()).as_int_or(99), 99);
    }

    #[test]
    fn as_bool_or_coerces_numbers_and_strings() {
        assert!(Value::Int(1).as_bool_or(false));
        assert!(!Value::Int(0).as_bool_or(true));
        assert!(!Value::String(String::new()).as_bool_or(true));
        assert!(Value::String("x".into()).as_bool_or(false));
        assert_eq!(Value::Null.as_bool_or(true), true);
    }

    #[test]
    fn from_vec_of_primitives() {
        let v: Value = vec![1i64, 2, 3].into();
        assert_eq!(
            v,
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }
}
