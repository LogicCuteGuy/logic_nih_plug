//! [`ValueTree`] — a hierarchical, serialisable, observable data tree.
//!
//! `ValueTree` is the Rust port of JUCE's `juce::ValueTree`. It is a reference-counted
//! tree of named properties and child nodes, designed for storing plugin state and
//! presets.
//!
//! ## Overview
//!
//! - A [`ValueTree`] is cheap to clone — clones share the same underlying data.
//! - Each tree has a *type* (an [`Identifier`]) and an ordered list of named
//!   properties and child trees.
//! - Mutations are observed through the [`ValueTreeListener`] trait.
//! - Mutations can be recorded on an [`UndoManager`] (when one is attached) for
//!   transactional undo/redo.
//!
//! `ValueTree` is **single-threaded** in the same way JUCE's value tree is: do not
//! share a tree between threads without external synchronisation.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::identifier::Identifier;
use crate::undo_manager::UndoManager;
use crate::value::Value;

/// RAII handle returned by [`ValueTree::add_listener`].
///
/// Dropping the handle removes the listener from the tree. If the underlying tree
/// has been dropped, dropping the handle is a no-op.
pub struct ListenerHandle {
    tree: Weak<ValueTreeInner>,
    id: usize,
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.tree.upgrade() {
            inner.remove_listener(self.id);
        }
    }
}

/// Receives notifications when a [`ValueTree`] changes.
///
/// All methods have no-op default implementations, so listeners only need to
/// override the events they care about. The trait requires [`Send`] + [`Sync`]
/// because listeners are stored inside an [`Arc`] that may be shared.
pub trait ValueTreeListener: Send + Sync {
    /// A property was added, removed or changed.
    fn value_tree_property_changed(&self, _tree: &ValueTree, _key: &Identifier) {}

    /// A child node was added to `parent` at `index`.
    fn value_tree_child_added(&self, _parent: &ValueTree, _child: &ValueTree, _index: usize) {}

    /// A child node was removed from `parent`. `index` is the position the child
    /// occupied before removal.
    fn value_tree_child_removed(&self, _parent: &ValueTree, _child: &ValueTree, _index: usize) {}

    /// A child was moved within its parent's children list.
    fn value_tree_child_order_changed(
        &self,
        _parent: &ValueTree,
        _old_index: usize,
        _new_index: usize,
    ) {
    }

    /// The tree was redirected to point at a different underlying node. This
    /// is used by JUCE when an undo recreates a deleted node — our implementation
    /// fires this whenever the inner arc is swapped (currently unused).
    fn value_tree_redirected(&self, _tree: &ValueTree) {}
}

/// Shared, reference-counted tree.
///
/// Cloning a `ValueTree` is cheap and produces a handle to the same underlying data.
/// Use [`ValueTree::create_copy`] to get a deep copy with an independent refcount.
#[derive(Clone)]
pub struct ValueTree {
    inner: Arc<ValueTreeInner>,
}

struct ValueTreeInner {
    state: Mutex<ValueTreeNode>,
    listeners: Mutex<Vec<ListenerEntry>>,
    undo_manager: Mutex<Option<crate::undo_manager::UndoManagerWeak>>,
    next_listener_id: AtomicUsize,
}

struct ListenerEntry {
    id: usize,
    listener: Arc<dyn ValueTreeListener>,
}

struct ValueTreeNode {
    type_name: Identifier,
    properties: Vec<(Identifier, Value)>,
    children: Vec<ValueTree>,
    parent: Option<Weak<ValueTreeInner>>,
}

impl ValueTreeInner {
    fn add_listener(&self, listener: Arc<dyn ValueTreeListener>) -> usize {
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        self.listeners.lock().unwrap().push(ListenerEntry { id, listener });
        id
    }

    fn remove_listener(&self, id: usize) {
        let mut listeners = self.listeners.lock().unwrap();
        if let Some(pos) = listeners.iter().position(|l| l.id == id) {
            listeners.swap_remove(pos);
        }
    }

    fn fire<F>(&self, f: F)
    where
        F: Fn(&dyn ValueTreeListener),
    {
        // Snapshot the listeners so we don't hold the lock while running user code.
        let snapshot: Vec<Arc<dyn ValueTreeListener>> = {
            let listeners = self.listeners.lock().unwrap();
            listeners.iter().map(|l| l.listener.clone()).collect()
        };
        for listener in &snapshot {
            f(&**listener);
        }
    }
}

impl ValueTree {
    /// Creates a new, empty tree of the given type.
    pub fn new(type_name: impl Into<Identifier>) -> Self {
        Self {
            inner: Arc::new(ValueTreeInner {
                state: Mutex::new(ValueTreeNode {
                    type_name: type_name.into(),
                    properties: Vec::new(),
                    children: Vec::new(),
                    parent: None,
                }),
                listeners: Mutex::new(Vec::new()),
                undo_manager: Mutex::new(None),
                next_listener_id: AtomicUsize::new(0),
            }),
        }
    }

    /// Returns `true`. (Kept for API parity with JUCE — under our `Arc` design a
    /// tree is always valid as long as a handle is alive.)
    pub fn is_valid(&self) -> bool {
        true
    }

    /// Returns the tree's type name.
    pub fn type_name(&self) -> Identifier {
        self.inner.state.lock().unwrap().type_name.clone()
    }

    /// Returns the parent tree, or `None` if this is a root.
    pub fn parent(&self) -> Option<ValueTree> {
        let state = self.inner.state.lock().unwrap();
        let weak = state.parent.as_ref()?;
        let inner = weak.upgrade()?;
        drop(state);
        Some(ValueTree { inner })
    }

    /// Returns the number of properties on this tree.
    pub fn num_properties(&self) -> usize {
        self.inner.state.lock().unwrap().properties.len()
    }

    /// Returns the property name at `index`, or `None` if out of range.
    pub fn get_property_name(&self, index: usize) -> Option<Identifier> {
        let state = self.inner.state.lock().unwrap();
        state.properties.get(index).map(|(k, _)| k.clone())
    }

    /// Returns a clone of the property value at `index`, or `None` if out of range.
    pub fn get_property_at(&self, index: usize) -> Option<Value> {
        let state = self.inner.state.lock().unwrap();
        state.properties.get(index).map(|(_, v)| v.clone())
    }

    /// Returns `true` if a property with `key` is set.
    pub fn has_property(&self, key: &Identifier) -> bool {
        let state = self.inner.state.lock().unwrap();
        state.properties.iter().any(|(k, _)| k == key)
    }

    /// Returns a clone of the property value, or `None` if not set.
    pub fn get_property(&self, key: &Identifier) -> Option<Value> {
        let state = self.inner.state.lock().unwrap();
        state
            .properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    /// Typed accessor — returns the property as `i64`, with `fallback` for missing
    /// or non-coercible values.
    pub fn get_int(&self, key: &Identifier, fallback: i64) -> i64 {
        self.get_property(key)
            .map(|v| v.as_int_or(fallback))
            .unwrap_or(fallback)
    }

    /// Typed accessor — returns the property as `f64`, with `fallback` for missing
    /// or non-coercible values.
    pub fn get_double(&self, key: &Identifier, fallback: f64) -> f64 {
        self.get_property(key)
            .map(|v| v.as_double_or(fallback))
            .unwrap_or(fallback)
    }

    /// Typed accessor — returns the property as `String`, with `fallback` for
    /// missing or non-string values.
    pub fn get_string(&self, key: &Identifier, fallback: &str) -> String {
        match self.get_property(key) {
            Some(Value::String(s)) => s,
            _ => fallback.to_owned(),
        }
    }

    /// Typed accessor — returns the property as `bool`, with `fallback` for missing
    /// or non-coercible values.
    pub fn get_bool(&self, key: &Identifier, fallback: bool) -> bool {
        self.get_property(key)
            .map(|v| v.as_bool_or(fallback))
            .unwrap_or(fallback)
    }

    /// Sets a property. If an undo manager is attached to this tree (via
    /// [`Self::set_undo_manager`]), the change is recorded; otherwise it is applied
    /// directly and listeners are notified.
    pub fn set_property(&self, key: impl Into<Identifier>, value: impl Into<Value>) {
        let key = key.into();
        let value = value.into();
        let attached = self.attached_undo_manager();
        let old_value = self.get_property(&key);
        if let Some(um) = attached {
            um.perform(crate::undo_manager::SetPropertyAction {
                tree: self.clone(),
                key: key.clone(),
                old_value,
                new_value: value,
            });
        } else {
            self.set_property_internal(key.clone(), value);
            self.fire_property_changed(&key);
        }
    }

    /// Like [`Self::set_property`] but explicitly targets a specific undo manager,
    /// regardless of any tree-attached manager.
    pub fn set_property_with(
        &self,
        key: impl Into<Identifier>,
        value: impl Into<Value>,
        undo_manager: &UndoManager,
    ) {
        let key = key.into();
        let value = value.into();
        let old_value = self.get_property(&key);
        undo_manager.perform(crate::undo_manager::SetPropertyAction {
            tree: self.clone(),
            key: key.clone(),
            old_value,
            new_value: value,
        });
    }

    /// Removes a property. Returns the previous value if it existed.
    pub fn remove_property(&self, key: impl Into<Identifier>) -> Option<Value> {
        let key = key.into();
        let attached = self.attached_undo_manager();
        let old_value = self.get_property(&key);
        if old_value.is_none() {
            return None;
        }
        let old_value = old_value.expect("checked is_none above");
        if let Some(um) = attached {
            um.perform(crate::undo_manager::RemovePropertyAction {
                tree: self.clone(),
                key: key.clone(),
                old_value: old_value.clone(),
            });
        } else {
            self.remove_property_internal(&key);
            self.fire_property_changed(&key);
        }
        Some(old_value)
    }

    /// Like [`Self::remove_property`] but explicitly targets a specific undo manager.
    pub fn remove_property_with(
        &self,
        key: impl Into<Identifier>,
        undo_manager: &UndoManager,
    ) -> Option<Value> {
        let key = key.into();
        let old_value = self.get_property(&key);
        if old_value.is_none() {
            return None;
        }
        let old_value = old_value.expect("checked is_none above");
        undo_manager.perform(crate::undo_manager::RemovePropertyAction {
            tree: self.clone(),
            key,
            old_value: old_value.clone(),
        });
        Some(old_value)
    }

    /// Removes all properties from this tree.
    pub fn clear_properties(&self) {
        let keys: Vec<Identifier> = {
            let state = self.inner.state.lock().unwrap();
            state.properties.iter().map(|(k, _)| k.clone()).collect()
        };
        for k in keys {
            self.remove_property(k);
        }
    }

    /// Returns the number of child trees.
    pub fn num_children(&self) -> usize {
        self.inner.state.lock().unwrap().children.len()
    }

    /// Returns the child at `index`, or `None` if out of range.
    pub fn get_child(&self, index: usize) -> Option<ValueTree> {
        let state = self.inner.state.lock().unwrap();
        state.children.get(index).cloned()
    }

    /// Returns the first child whose `type_name()` matches `type_name`.
    pub fn get_child_with_name(&self, type_name: &Identifier) -> Option<ValueTree> {
        let state = self.inner.state.lock().unwrap();
        state
            .children
            .iter()
            .find(|c| {
                let cs = c.inner.state.lock().unwrap();
                &cs.type_name == type_name
            })
            .cloned()
    }

    /// Returns the index of `child` within this tree's children, or `None` if it is
    /// not a child. Comparison is by underlying node identity.
    pub fn index_of_child(&self, child: &ValueTree) -> Option<usize> {
        let state = self.inner.state.lock().unwrap();
        state
            .children
            .iter()
            .position(|c| Arc::ptr_eq(&c.inner, &child.inner))
    }

    /// Returns `true` if `self` is the same tree as `other` (same underlying node).
    pub fn is_same_as(&self, other: &ValueTree) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// Returns `true` if `self` is `parent` or one of its descendants.
    pub fn is_a_child_of(&self, parent: &ValueTree) -> bool {
        let mut current = self.parent();
        while let Some(p) = current {
            if p.is_same_as(parent) {
                return true;
            }
            current = p.parent();
        }
        false
    }

    /// Adds `child` as a child of this tree at `index`. Detaches the child from any
    /// previous parent.
    pub fn add_child(&self, child: ValueTree, index: usize) {
        self.detach_from_parent(&child);
        let target_index = {
            let mut state = self.inner.state.lock().unwrap();
            let idx = index.min(state.children.len());
            state.children.insert(idx, child.clone());
            idx
        };
        self.set_child_parent(&child);
        self.fire_child_added(&child, target_index);
    }

    /// Like [`Self::add_child`] but records the change on `undo_manager`.
    pub fn add_child_with(&self, child: ValueTree, index: usize, undo_manager: &UndoManager) {
        undo_manager.perform(crate::undo_manager::AddChildAction {
            parent: self.clone(),
            child,
            index,
        });
    }

    /// Removes the child at `index` and returns it. Returns `None` if out of range.
    pub fn remove_child(&self, index: usize) -> Option<ValueTree> {
        let removed = {
            let mut state = self.inner.state.lock().unwrap();
            if index >= state.children.len() {
                return None;
            }
            let removed = state.children.remove(index);
            Some(removed)
        };
        if let Some(ref child) = removed {
            self.clear_child_parent(child);
            self.fire_child_removed(child, index);
        }
        removed
    }

    /// Like [`Self::remove_child`] but records the change on `undo_manager`.
    pub fn remove_child_with(
        &self,
        index: usize,
        undo_manager: &UndoManager,
    ) -> Option<ValueTree> {
        let child = self.get_child(index)?;
        undo_manager.perform(crate::undo_manager::RemoveChildAction {
            parent: self.clone(),
            child,
            index,
        });
        Some(self.get_child(index).unwrap_or_else(|| {
            // After removal, the tree at index is what was index+1 before.
            // But since we recorded the removed child by identity, this is moot.
            // Return the originally-captured child for caller convenience.
            unreachable!("remove_child_with should be called via undo path")
        }))
    }

    /// Moves the child at `old_index` to `new_index`, leaving the rest of the
    /// children in their original relative order.
    pub fn move_child(&self, old_index: usize, new_index: usize) {
        let len = self.num_children();
        if old_index >= len || new_index >= len || old_index == new_index {
            return;
        }
        let moved = {
            let mut state = self.inner.state.lock().unwrap();
            let moved = state.children.remove(old_index);
            let target = new_index.min(state.children.len());
            state.children.insert(target, moved.clone());
            (moved, target)
        };
        self.fire_child_order_changed(old_index, moved.1);
        let _ = moved;
    }

    /// Registers `listener` to be notified of changes. Drop the returned handle to
    /// unregister.
    pub fn add_listener(&self, listener: Arc<dyn ValueTreeListener>) -> ListenerHandle {
        let id = self.inner.add_listener(listener);
        ListenerHandle {
            tree: Arc::downgrade(&self.inner),
            id,
        }
    }

    /// Attaches an undo manager to this tree. All subsequent mutations that don't
    /// pass an explicit manager will be recorded on this one.
    ///
    /// Pass `None` to detach. Passing `Some(&manager)` while a different manager is
    /// already attached replaces it.
    pub fn set_undo_manager(&self, manager: Option<&UndoManager>) {
        let mut slot = self.inner.undo_manager.lock().unwrap();
        *slot = manager.map(UndoManager::downgrade);
    }

    /// Returns the currently attached undo manager, if any.
    pub fn undo_manager(&self) -> Option<UndoManager> {
        self.attached_undo_manager()
    }

    /// Creates a deep copy of this tree with independent refcounts. Listeners and
    /// undo-manager attachments are not copied.
    pub fn create_copy(&self) -> ValueTree {
        let copy = ValueTree::new(self.type_name());
        {
            let src = self.inner.state.lock().unwrap();
            let mut dst = copy.inner.state.lock().unwrap();
            dst.properties = src.properties.clone();
            dst.children = src
                .children
                .iter()
                .map(|c| {
                    let child_copy = c.create_copy();
                    self.set_child_parent_external(&child_copy, &copy.inner);
                    child_copy
                })
                .collect();
        }
        copy
    }

    /// Returns `true` if `self` and `other` have the same type and an equal set of
    /// properties and children (recursively).
    pub fn is_identical_to(&self, other: &ValueTree) -> bool {
        if Arc::ptr_eq(&self.inner, &other.inner) {
            return true;
        }
        let s = self.inner.state.lock().unwrap();
        let o = other.inner.state.lock().unwrap();
        if s.type_name != o.type_name {
            return false;
        }
        if s.properties != o.properties {
            return false;
        }
        if s.children.len() != o.children.len() {
            return false;
        }
        for (a, b) in s.children.iter().zip(o.children.iter()) {
            if !a.is_identical_to(b) {
                return false;
            }
        }
        true
    }

    // ------------------------------------------------------------------
    // Internal helpers used by undo actions.
    // ------------------------------------------------------------------

    pub(crate) fn set_property_internal(&self, key: Identifier, value: Value) {
        let mut state = self.inner.state.lock().unwrap();
        if let Some((_, existing)) = state.properties.iter_mut().find(|(k, _)| k == &key) {
            *existing = value;
        } else {
            state.properties.push((key, value));
        }
    }

    pub(crate) fn remove_property_internal(&self, key: &Identifier) {
        let mut state = self.inner.state.lock().unwrap();
        state.properties.retain(|(k, _)| k != key);
    }

    pub(crate) fn add_child_internal(&self, child: ValueTree, index: usize) {
        self.detach_from_parent(&child);
        let mut state = self.inner.state.lock().unwrap();
        let idx = index.min(state.children.len());
        state.children.insert(idx, child);
        drop(state);
        self.set_child_parent_external(
            &self
                .get_child(idx)
                .expect("child was just inserted"),
            &self.inner,
        );
    }

    pub(crate) fn remove_child_internal(&self, child: &ValueTree) -> Option<usize> {
        let mut state = self.inner.state.lock().unwrap();
        let pos = state
            .children
            .iter()
            .position(|c| Arc::ptr_eq(&c.inner, &child.inner))?;
        state.children.remove(pos);
        drop(state);
        self.clear_child_parent(child);
        Some(pos)
    }

    pub(crate) fn fire_property_changed(&self, key: &Identifier) {
        let inner = self.inner.clone();
        let key = key.clone();
        inner.fire(|l| l.value_tree_property_changed(
            // SAFETY-ish: we recreate the ValueTree view from the same inner Arc.
            &ValueTree { inner: Arc::clone(&inner) },
            &key,
        ));
    }

    pub(crate) fn fire_child_added(&self, child: &ValueTree, index: usize) {
        let inner = self.inner.clone();
        let child_clone = child.clone();
        inner.fire(|l| {
            let tree = ValueTree { inner: inner.clone() };
            l.value_tree_child_added(&tree, &child_clone, index);
        });
    }

    pub(crate) fn fire_child_removed(&self, child: &ValueTree, index: usize) {
        let inner = self.inner.clone();
        let child_clone = child.clone();
        inner.fire(|l| {
            let tree = ValueTree { inner: inner.clone() };
            l.value_tree_child_removed(&tree, &child_clone, index);
        });
    }

    pub(crate) fn fire_child_order_changed(&self, old_index: usize, new_index: usize) {
        let inner = self.inner.clone();
        inner.fire(|l| {
            let tree = ValueTree { inner: inner.clone() };
            l.value_tree_child_order_changed(&tree, old_index, new_index);
        });
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    fn attached_undo_manager(&self) -> Option<UndoManager> {
        let slot = self.inner.undo_manager.lock().unwrap();
        slot.as_ref()
            .and_then(|w| w.upgrade())
            .map(UndoManager::from_state)
    }

    fn detach_from_parent(&self, child: &ValueTree) {
        if let Some(parent_weak) = child.inner.state.lock().unwrap().parent.clone() {
            if let Some(parent_inner) = parent_weak.upgrade() {
                if !Arc::ptr_eq(&parent_inner, &self.inner) {
                    let mut state = parent_inner.state.lock().unwrap();
                    state.children.retain(|c| !Arc::ptr_eq(&c.inner, &child.inner));
                }
            }
        }
    }

    fn set_child_parent(&self, child: &ValueTree) {
        let mut state = child.inner.state.lock().unwrap();
        state.parent = Some(Arc::downgrade(&self.inner));
    }

    fn set_child_parent_external(&self, child: &ValueTree, parent: &Arc<ValueTreeInner>) {
        let mut state = child.inner.state.lock().unwrap();
        state.parent = Some(Arc::downgrade(parent));
    }

    fn clear_child_parent(&self, child: &ValueTree) {
        let mut state = child.inner.state.lock().unwrap();
        state.parent = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn new_tree_has_type_and_no_children() {
        let tree = ValueTree::new("Root");
        assert_eq!(tree.type_name().as_str(), "Root");
        assert_eq!(tree.num_properties(), 0);
        assert_eq!(tree.num_children(), 0);
        assert!(tree.parent().is_none());
    }

    #[test]
    fn property_round_trip() {
        let tree = ValueTree::new("Root");
        tree.set_property("gain", 0.5_f64);
        assert!(tree.has_property(&Identifier::new("gain")));
        assert_eq!(tree.get_double(&Identifier::new("gain"), 0.0), 0.5);
        assert_eq!(tree.get_int(&Identifier::new("gain"), 0), 0);
        assert_eq!(tree.get_string(&Identifier::new("gain"), "x"), "x");
        assert!(tree.get_bool(&Identifier::new("gain"), false));
    }

    #[test]
    fn remove_property_returns_old() {
        let tree = ValueTree::new("Root");
        tree.set_property("k", 42_i64);
        let old = tree.remove_property("k");
        assert_eq!(old, Some(Value::Int(42)));
        assert!(!tree.has_property(&Identifier::new("k")));
    }

    #[test]
    fn add_and_get_child() {
        let parent = ValueTree::new("Parent");
        let child = ValueTree::new("Child");
        parent.add_child(child.clone(), 0);
        assert_eq!(parent.num_children(), 1);
        assert_eq!(parent.get_child(0).unwrap().type_name().as_str(), "Child");
        assert!(child.parent().unwrap().is_same_as(&parent));
    }

    #[test]
    fn moving_child_keeps_identity() {
        let parent = ValueTree::new("Parent");
        let a = ValueTree::new("A");
        let b = ValueTree::new("B");
        let c = ValueTree::new("C");
        parent.add_child(a.clone(), 0);
        parent.add_child(b.clone(), 1);
        parent.add_child(c.clone(), 2);
        parent.move_child(0, 2);
        assert!(parent.get_child(0).unwrap().is_same_as(&b));
        assert!(parent.get_child(1).unwrap().is_same_as(&c));
        assert!(parent.get_child(2).unwrap().is_same_as(&a));
    }

    #[test]
    fn detach_moves_child_between_parents() {
        let p1 = ValueTree::new("P1");
        let p2 = ValueTree::new("P2");
        let c = ValueTree::new("C");
        p1.add_child(c.clone(), 0);
        p2.add_child(c.clone(), 0);
        assert_eq!(p1.num_children(), 0);
        assert_eq!(p2.num_children(), 1);
        assert!(c.parent().unwrap().is_same_as(&p2));
    }

    #[test]
    fn listener_receives_property_change() {
        let tree = ValueTree::new("Root");
        let count = Arc::new(AtomicUsize::new(0));
        let count_in_listener = count.clone();
        let listener = Arc::new(RecordingListener {
            property_changes: count_in_listener,
        });
        let _handle = tree.add_listener(listener);
        tree.set_property("a", 1_i64);
        tree.set_property("b", 2_i64);
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn listener_receives_child_events() {
        let tree = ValueTree::new("Root");
        let added = Arc::new(AtomicUsize::new(0));
        let removed = Arc::new(AtomicUsize::new(0));
        let listener = Arc::new(ChildCountListener {
            added: added.clone(),
            removed: removed.clone(),
        });
        let _handle = tree.add_listener(listener);
        let c = ValueTree::new("C");
        tree.add_child(c.clone(), 0);
        assert_eq!(added.load(Ordering::SeqCst), 1);
        tree.remove_child(0);
        assert_eq!(removed.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn handle_drop_removes_listener() {
        let tree = ValueTree::new("Root");
        let count = Arc::new(AtomicUsize::new(0));
        let count_in_listener = count.clone();
        let listener = Arc::new(RecordingListener {
            property_changes: count_in_listener,
        });
        let handle = tree.add_listener(listener);
        tree.set_property("a", 1_i64);
        assert_eq!(count.load(Ordering::SeqCst), 1);
        drop(handle);
        tree.set_property("a", 2_i64);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn is_a_child_of_walks_chain() {
        let root = ValueTree::new("Root");
        let mid = ValueTree::new("Mid");
        let leaf = ValueTree::new("Leaf");
        root.add_child(mid.clone(), 0);
        mid.add_child(leaf.clone(), 0);
        assert!(leaf.is_a_child_of(&root));
        assert!(leaf.is_a_child_of(&mid));
        assert!(mid.is_a_child_of(&root));
        assert!(!root.is_a_child_of(&leaf));
        assert!(!mid.is_a_child_of(&leaf));
    }

    #[test]
    fn create_copy_is_independent() {
        let original = ValueTree::new("Root");
        original.set_property("k", 7_i64);
        let child = ValueTree::new("C");
        original.add_child(child, 0);
        let copy = original.create_copy();
        assert!(!copy.is_same_as(&original));
        assert_eq!(copy.num_properties(), 1);
        assert_eq!(copy.num_children(), 1);
        copy.set_property("k", 99_i64);
        assert_eq!(original.get_int(&Identifier::new("k"), 0), 7);
        assert_eq!(copy.get_int(&Identifier::new("k"), 0), 99);
    }

    #[test]
    fn is_identical_to_recursive() {
        let a = ValueTree::new("Root");
        a.set_property("n", 5_i64);
        let b = ValueTree::new("Root");
        b.set_property("n", 5_i64);
        assert!(a.is_identical_to(&b));
        b.set_property("n", 6_i64);
        assert!(!a.is_identical_to(&b));
    }

    struct RecordingListener {
        property_changes: Arc<AtomicUsize>,
    }

    impl ValueTreeListener for RecordingListener {
        fn value_tree_property_changed(&self, _tree: &ValueTree, _key: &Identifier) {
            self.property_changes.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct ChildCountListener {
        added: Arc<AtomicUsize>,
        removed: Arc<AtomicUsize>,
    }

    impl ValueTreeListener for ChildCountListener {
        fn value_tree_child_added(
            &self,
            _parent: &ValueTree,
            _child: &ValueTree,
            _index: usize,
        ) {
            self.added.fetch_add(1, Ordering::SeqCst);
        }
        fn value_tree_child_removed(
            &self,
            _parent: &ValueTree,
            _child: &ValueTree,
            _index: usize,
        ) {
            self.removed.fetch_add(1, Ordering::SeqCst);
        }
    }
}
