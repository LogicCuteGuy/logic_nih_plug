//! [`CachedValue<T>`] — typed binding to a [`ValueTree`] property.
//!
//! `CachedValue<T>` keeps a typed Rust variable synchronised with a named property
//! on a [`ValueTree`]. When the tree's property changes, the cached value updates.
//! When you set the cached value, it writes back to the tree.
//!
//! This mirrors JUCE's `CachedValue<T>` template. Use it when you want a plain
//! `f64` / `i64` / `String` / `bool` to track a property without manually wiring up
//! the listener boilerplate.

use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::identifier::Identifier;
use crate::value::Value;
use crate::value_tree::{ListenerHandle, ValueTree, ValueTreeListener};

/// Conversion between [`Value`] and a cached type.
///
/// Implement this for your own types if you want to use them with [`CachedValue`].
pub trait CachedValueTrait: Sized + Clone {
    /// Builds `Self` from a tree-stored [`Value`]. Implementations should be tolerant
    /// of missing or non-matching variants — fall back to a sensible default rather
    /// than panicking.
    fn from_value(v: &Value) -> Self;

    /// Returns the [`Value`] representation of `self`.
    fn to_value(&self) -> Value;
}

impl CachedValueTrait for i64 {
    fn from_value(v: &Value) -> Self {
        v.as_int_or(0)
    }
    fn to_value(&self) -> Value {
        Value::Int(*self)
    }
}

impl CachedValueTrait for i32 {
    fn from_value(v: &Value) -> Self {
        v.as_int_or(0) as i32
    }
    fn to_value(&self) -> Value {
        Value::Int(*self as i64)
    }
}

impl CachedValueTrait for u32 {
    fn from_value(v: &Value) -> Self {
        v.as_int_or(0) as u32
    }
    fn to_value(&self) -> Value {
        Value::Int(*self as i64)
    }
}

impl CachedValueTrait for usize {
    fn from_value(v: &Value) -> Self {
        v.as_int_or(0) as usize
    }
    fn to_value(&self) -> Value {
        Value::Int(*self as i64)
    }
}

impl CachedValueTrait for f64 {
    fn from_value(v: &Value) -> Self {
        v.as_double_or(0.0)
    }
    fn to_value(&self) -> Value {
        Value::Double(*self)
    }
}

impl CachedValueTrait for f32 {
    fn from_value(v: &Value) -> Self {
        v.as_double_or(0.0) as f32
    }
    fn to_value(&self) -> Value {
        Value::Double(*self as f64)
    }
}

impl CachedValueTrait for bool {
    fn from_value(v: &Value) -> Self {
        v.as_bool_or(false)
    }
    fn to_value(&self) -> Value {
        Value::Bool(*self)
    }
}

impl CachedValueTrait for String {
    fn from_value(v: &Value) -> Self {
        match v {
            Value::String(s) => s.clone(),
            _ => String::new(),
        }
    }
    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

/// Bind a typed Rust variable to a property on a [`ValueTree`].
///
/// The cached value mirrors the tree: reads via [`get`](Self::get) always return the
/// latest tree value, writes via [`set`](Self::set) update the tree. If the
/// property is removed from the tree, [`get`](Self::get) returns the `default`
/// supplied at construction.
///
/// # Example
///
/// ```rust
/// use logic_nih_plug_data::{CachedValue, ValueTree};
///
/// let tree = ValueTree::new("Synth");
/// let gain = CachedValue::<f64>::new(&tree, "gain", 1.0);
/// assert_eq!(gain.get(), 1.0);
/// gain.set(0.25);
/// assert_eq!(tree.get_double(&"gain".into(), 0.0), 0.25);
/// assert_eq!(gain.get(), 0.25);
/// tree.remove_property("gain");
/// assert_eq!(gain.get(), 1.0); // Falls back to default.
/// ```
pub struct CachedValue<T>
where
    T: CachedValueTrait + Send + 'static,
{
    tree: ValueTree,
    property: Identifier,
    state: Arc<Mutex<CachedValueState<T>>>,
    _handle: ListenerHandle,
}

struct CachedValueState<T> {
    value: Option<T>,
    default: T,
}

struct CachedValueListener<T> {
    state: Arc<Mutex<CachedValueState<T>>>,
    property: Identifier,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> CachedValueListener<T> {
    fn new(state: Arc<Mutex<CachedValueState<T>>>, property: Identifier) -> Self {
        Self {
            state,
            property,
            _phantom: PhantomData,
        }
    }
}

impl<T> ValueTreeListener for CachedValueListener<T>
where
    T: CachedValueTrait + Send + 'static,
{
    fn value_tree_property_changed(&self, tree: &ValueTree, key: &Identifier) {
        if key != &self.property {
            return;
        }
        let mut state = self.state.lock().expect("cached value poisoned");
        match tree.get_property(&self.property) {
            Some(v) => state.value = Some(T::from_value(&v)),
            None => state.value = None,
        }
    }
}

impl<T> CachedValue<T>
where
    T: CachedValueTrait + Send + 'static,
{
    /// Creates a new `CachedValue` bound to `property` on `tree`. If the tree does
    /// not yet have a value for that property, the cached value starts at `default`.
    pub fn new(tree: &ValueTree, property: impl Into<Identifier>, default: T) -> Self {
        let property = property.into();
        let state = Arc::new(Mutex::new(CachedValueState {
            value: None,
            default: default.clone(),
        }));

        // Initial sync from the tree (if it already has a value).
        if let Some(v) = tree.get_property(&property) {
            let mut s = state.lock().expect("cached value poisoned");
            s.value = Some(T::from_value(&v));
        }

        let listener = Arc::new(CachedValueListener::new(state.clone(), property.clone()));
        let _handle = tree.add_listener(listener);

        Self {
            tree: tree.clone(),
            property,
            state,
            _handle,
        }
    }

    /// Returns the current value of the cached variable. Falls back to the
    /// `default` passed to [`new`](Self::new) when the tree has no value for the
    /// property.
    pub fn get(&self) -> T {
        let s = self.state.lock().expect("cached value poisoned");
        s.value.clone().unwrap_or_else(|| s.default.clone())
    }

    /// Sets the cached variable and writes the new value to the tree.
    pub fn set(&self, value: T) {
        let to_store = value.clone();
        {
            let mut s = self.state.lock().expect("cached value poisoned");
            s.value = Some(value);
        }
        self.tree
            .set_property(self.property.clone(), to_store.to_value());
    }

    /// Re-reads the tree and updates the cached variable. Use after directly
    /// manipulating the tree (e.g. inside a [`crate::UndoManager`] transaction).
    pub fn force_update(&self) {
        let mut s = self.state.lock().expect("cached value poisoned");
        match self.tree.get_property(&self.property) {
            Some(v) => s.value = Some(T::from_value(&v)),
            None => s.value = None,
        }
    }

    /// Returns `true` if the cached variable is currently using its `default`
    /// value because the tree has no value for the property.
    pub fn is_using_default(&self) -> bool {
        let s = self.state.lock().expect("cached value poisoned");
        s.value.is_none()
    }

    /// Returns the property identifier this cached value is bound to.
    pub fn property(&self) -> &Identifier {
        &self.property
    }

    /// Returns the tree this cached value is bound to.
    pub fn tree(&self) -> &ValueTree {
        &self.tree
    }

    /// Returns the default value supplied at construction.
    pub fn default_value(&self) -> T {
        let s = self.state.lock().expect("cached value poisoned");
        s.default.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_value_used_when_property_absent() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<f64>::new(&tree, "gain", 1.0);
        assert_eq!(cv.get(), 1.0);
        assert!(cv.is_using_default());
    }

    #[test]
    fn set_propagates_to_tree() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<f64>::new(&tree, "gain", 1.0);
        cv.set(0.5);
        assert_eq!(tree.get_double(&"gain".into(), 0.0), 0.5);
        assert_eq!(cv.get(), 0.5);
        assert!(!cv.is_using_default());
    }

    #[test]
    fn external_tree_change_updates_cache() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<f64>::new(&tree, "gain", 1.0);
        tree.set_property("gain", 0.75_f64);
        assert_eq!(cv.get(), 0.75);
        assert!(!cv.is_using_default());
    }

    #[test]
    fn removing_property_falls_back_to_default() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<i64>::new(&tree, "count", 42);
        cv.set(7);
        assert_eq!(cv.get(), 7);
        tree.remove_property("count");
        assert_eq!(cv.get(), 42);
        assert!(cv.is_using_default());
    }

    #[test]
    fn bool_conversion() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<bool>::new(&tree, "on", false);
        tree.set_property("on", true);
        assert!(cv.get());
        cv.set(false);
        assert!(!tree.get_bool(&"on".into(), true));
    }

    #[test]
    fn string_conversion() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<String>::new(&tree, "name", "default".to_owned());
        tree.set_property("name", "hello".to_owned());
        assert_eq!(cv.get(), "hello");
    }

    #[test]
    fn int_coercion_from_double() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<i64>::new(&tree, "n", 0);
        tree.set_property("n", 3.7_f64);
        assert_eq!(cv.get(), 3);
    }

    #[test]
    fn force_update_pulls_from_tree() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<f64>::new(&tree, "gain", 0.0);
        // Modify the tree directly without going through `set`.
        tree.set_property("gain", 0.1_f64);
        // The listener should already have updated cv, but this verifies force_update.
        cv.force_update();
        assert_eq!(cv.get(), 0.1);
    }

    #[test]
    fn handle_drop_disconnects_listener() {
        let tree = ValueTree::new("Root");
        let cv = CachedValue::<f64>::new(&tree, "gain", 0.0);
        cv.set(0.5);
        // Drop the cached value explicitly. The tree remains usable.
        drop(cv);
        tree.set_property("gain", 0.9_f64);
        assert_eq!(tree.get_double(&"gain".into(), 0.0), 0.9);
    }
}
