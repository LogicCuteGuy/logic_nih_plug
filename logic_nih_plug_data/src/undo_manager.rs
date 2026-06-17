//! [`UndoManager`] — transactional undo/redo for [`ValueTree`] changes.
//!
//! [`UndoManager`] records performed [`UndoableAction`]s and lets you undo and redo
//! them. Multiple actions can be grouped together with [`begin_transaction`] /
//! [`end_transaction`] so they undo as a single step.
//!
//! [`UndoManager`] is `Send + Sync` so it can be stored inside cross-thread listeners,
//! at the cost of a single mutex acquisition per recorded operation.
//!
//! [`begin_transaction`]: UndoManager::begin_transaction
//! [`end_transaction`]: UndoManager::end_transaction

use std::sync::{Arc, Mutex, Weak};

use crate::identifier::Identifier;
use crate::value::Value;
use crate::value_tree::ValueTree;

/// An action that can be performed on a [`ValueTree`] and later undone.
///
/// `perform` is called when the action is first executed; `undo` is called when the
/// user reverses it.
pub trait UndoableAction: Send + 'static {
    /// Applies the action. Returns `false` if the action could not be performed
    /// (in which case it will not be recorded).
    fn perform(&mut self, manager: &UndoManager) -> bool;

    /// Reverses the action. Returns `false` if the action could not be undone.
    fn undo(&mut self, manager: &UndoManager) -> bool;

    /// Approximate memory cost, used to enforce
    /// [`set_max_undo_storage_size`](UndoManager::set_max_undo_storage_size).
    fn get_size(&self) -> i32 {
        1
    }

    /// Short human-readable description, shown by `undo_description` /
    /// `redo_description`.
    fn get_description(&self) -> Option<&str> {
        None
    }
}

/// Records performed [`UndoableAction`]s and supports undo/redo with optional
/// grouping into transactions.
///
/// # Example
///
/// ```rust
/// use logic_nih_plug_data::{UndoManager, UndoableAction, ValueTree, Value};
///
/// let tree = ValueTree::new("Root");
/// let undo = UndoManager::new();
/// tree.set_property_with("gain", 0.5_f64, &undo);
/// assert_eq!(tree.get_double(&"gain".into(), 0.0), 0.5);
/// assert!(undo.undo());
/// assert!(!tree.has_property(&"gain".into()));
/// assert!(undo.redo());
/// assert_eq!(tree.get_double(&"gain".into(), 0.0), 0.5);
/// ```
pub struct UndoManager {
    state: Arc<Mutex<UndoManagerState>>,
}

#[derive(Default)]
pub(crate) struct UndoManagerState {
    undo_stack: Vec<Undoable>,
    redo_stack: Vec<Undoable>,
    transaction_stack: Vec<Transaction>,
    max_storage: i32,
    current_storage: i32,
}

/// A weak reference to an [`UndoManager`]'s internal state, used by
/// [`crate::ValueTree::set_undo_manager`] to track an attached manager.
pub(crate) type UndoManagerWeak = Weak<Mutex<UndoManagerState>>;

enum Undoable {
    Action(Box<dyn UndoableAction>),
    Group(Transaction),
}

struct Transaction {
    name: Option<String>,
    actions: Vec<Box<dyn UndoableAction>>,
}

impl Undoable {
    fn perform(&mut self, manager: &UndoManager) -> bool {
        match self {
            Undoable::Action(a) => a.perform(manager),
            Undoable::Group(g) => {
                let mut ok = true;
                for a in g.actions.iter_mut() {
                    if !a.perform(manager) {
                        ok = false;
                    }
                }
                ok
            }
        }
    }

    fn undo(&mut self, manager: &UndoManager) -> bool {
        match self {
            Undoable::Action(a) => a.undo(manager),
            Undoable::Group(g) => {
                let mut ok = true;
                for a in g.actions.iter_mut().rev() {
                    if !a.undo(manager) {
                        ok = false;
                    }
                }
                ok
            }
        }
    }

    fn description(&self) -> Option<String> {
        match self {
            Undoable::Action(a) => a.get_description().map(str::to_owned),
            Undoable::Group(g) => g.name.clone(),
        }
    }

    fn size(&self) -> i32 {
        match self {
            Undoable::Action(a) => a.get_size(),
            Undoable::Group(g) => g.actions.iter().map(|a| a.get_size()).sum(),
        }
    }
}

impl Clone for UndoManager {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoManager {
    /// Creates a new, empty undo manager.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(UndoManagerState::default())),
        }
    }

    /// Performs `action` and records it. If a transaction is active, the action is
    /// appended to it instead of the undo stack.
    ///
    /// Returns `true` if the action was performed and recorded, `false` if it
    /// returned `false` from `perform`.
    pub fn perform<A: UndoableAction>(&self, mut action: A) -> bool {
        if !action.perform(self) {
            return false;
        }
        let size = action.get_size();
        let boxed: Box<dyn UndoableAction> = Box::new(action);
        let mut state = self.state.lock().expect("undo manager poisoned");
        if let Some(txn) = state.transaction_stack.last_mut() {
            txn.actions.push(boxed);
            return true;
        }
        state.redo_stack.clear();
        state.undo_stack.push(Undoable::Action(boxed));
        state.current_storage += size;
        state.enforce_storage_limit();
        true
    }

    /// Undoes the most recent action. Returns `true` if anything was undone.
    pub fn undo(&self) -> bool {
        let mut state = self.state.lock().expect("undo manager poisoned");
        let Some(mut top) = state.undo_stack.pop() else {
            return false;
        };
        if !top.undo(self) {
            // Drop the action — JUCE would record it in an "aborted" stack; we don't.
            return false;
        }
        state.redo_stack.push(top);
        true
    }

    /// Redoes the most recently undone action. Returns `true` if anything was redone.
    pub fn redo(&self) -> bool {
        let mut state = self.state.lock().expect("undo manager poisoned");
        let Some(mut top) = state.redo_stack.pop() else {
            return false;
        };
        if !top.perform(self) {
            return false;
        }
        state.undo_stack.push(top);
        true
    }

    /// Begins a new transaction. Subsequent actions performed on this manager will be
    /// grouped into it until [`end_transaction`] is called. Transactions can be
    /// nested.
    ///
    /// [`end_transaction`]: Self::end_transaction
    pub fn begin_transaction(&self, name: impl Into<String>) {
        let mut state = self.state.lock().expect("undo manager poisoned");
        state.transaction_stack.push(Transaction {
            name: Some(name.into()),
            actions: Vec::new(),
        });
    }

    /// Closes the current transaction and pushes it onto the undo stack. Returns
    /// `false` if no transaction was active, or if the transaction was empty.
    pub fn end_transaction(&self) -> bool {
        let mut state = self.state.lock().expect("undo manager poisoned");
        let Some(txn) = state.transaction_stack.pop() else {
            return false;
        };
        if txn.actions.is_empty() {
            return false;
        }
        let size: i32 = txn.actions.iter().map(|a| a.get_size()).sum();
        state.undo_stack.push(Undoable::Group(txn));
        state.redo_stack.clear();
        state.current_storage += size;
        state.enforce_storage_limit();
        true
    }

    /// Discards the current transaction and any actions it has collected.
    pub fn cancel_transaction(&self) -> bool {
        let mut state = self.state.lock().expect("undo manager poisoned");
        state.transaction_stack.pop().is_some()
    }

    /// Returns `true` if a transaction is currently active.
    pub fn is_in_transaction(&self) -> bool {
        !self
            .state
            .lock()
            .expect("undo manager poisoned")
            .transaction_stack
            .is_empty()
    }

    /// Returns the name of the innermost active transaction, if any.
    pub fn current_transaction_name(&self) -> Option<String> {
        self.state
            .lock()
            .expect("undo manager poisoned")
            .transaction_stack
            .last()
            .and_then(|t| t.name.clone())
    }

    /// Returns `true` if there is an action that can be undone.
    pub fn can_undo(&self) -> bool {
        !self
            .state
            .lock()
            .expect("undo manager poisoned")
            .undo_stack
            .is_empty()
    }

    /// Returns `true` if there is an action that can be redone.
    pub fn can_redo(&self) -> bool {
        !self
            .state
            .lock()
            .expect("undo manager poisoned")
            .redo_stack
            .is_empty()
    }

    /// Returns the description of the next action to undo, if any.
    pub fn undo_description(&self) -> Option<String> {
        self.state
            .lock()
            .expect("undo manager poisoned")
            .undo_stack
            .last()
            .and_then(|u| u.description())
    }

    /// Returns the description of the next action to redo, if any.
    pub fn redo_description(&self) -> Option<String> {
        self.state
            .lock()
            .expect("undo manager poisoned")
            .redo_stack
            .last()
            .and_then(|u| u.description())
    }

    /// Returns the number of undoable actions.
    pub fn undo_count(&self) -> usize {
        self.state
            .lock()
            .expect("undo manager poisoned")
            .undo_stack
            .len()
    }

    /// Returns the number of redoable actions.
    pub fn redo_count(&self) -> usize {
        self.state
            .lock()
            .expect("undo manager poisoned")
            .redo_stack
            .len()
    }

    /// Clears all undo and redo history, dropping any actions. Active transactions
    /// are also cancelled.
    pub fn clear_history(&self) {
        let mut state = self.state.lock().expect("undo manager poisoned");
        state.undo_stack.clear();
        state.redo_stack.clear();
        state.transaction_stack.clear();
        state.current_storage = 0;
    }

    /// Sets the maximum storage cost for the undo stack, in the same units as
    /// [`UndoableAction::get_size`]. Older actions are discarded first when the limit
    /// is exceeded. A non-positive value disables the limit.
    pub fn set_max_undo_storage_size(&self, limit: i32) {
        let mut state = self.state.lock().expect("undo manager poisoned");
        state.max_storage = limit;
        state.enforce_storage_limit();
    }

    /// Constructs an `UndoManager` from a shared state Arc. Used by
    /// [`crate::ValueTree::set_undo_manager`] when re-attaching a tree after a weak
    /// upgrade.
    pub(crate) fn from_state(state: Arc<Mutex<UndoManagerState>>) -> Self {
        Self { state }
    }

    /// Returns a `Weak` handle to the manager's state, for tree attachment.
    pub(crate) fn downgrade(&self) -> Weak<Mutex<UndoManagerState>> {
        Arc::downgrade(&self.state)
    }
}

impl UndoManagerState {
    fn enforce_storage_limit(&mut self) {
        if self.max_storage <= 0 {
            return;
        }
        while self.current_storage > self.max_storage && !self.undo_stack.is_empty() {
            let removed = self.undo_stack.remove(0);
            self.current_storage -= removed.size();
        }
    }
}

// ----------------------------------------------------------------------------
// Concrete actions emitted by [`crate::ValueTree`].
// ----------------------------------------------------------------------------

/// Recorded by [`ValueTree::set_property_with`] and [`ValueTree::set_property`] when
/// an undo manager is attached.
pub struct SetPropertyAction {
    pub(crate) tree: ValueTree,
    pub(crate) key: Identifier,
    pub(crate) old_value: Option<Value>,
    pub(crate) new_value: Value,
}

impl UndoableAction for SetPropertyAction {
    fn perform(&mut self, _manager: &UndoManager) -> bool {
        self.tree
            .set_property_internal(self.key.clone(), self.new_value.clone());
        self.tree.fire_property_changed(&self.key);
        true
    }
    fn undo(&mut self, _manager: &UndoManager) -> bool {
        match self.old_value.take() {
            Some(old) => self
                .tree
                .set_property_internal(self.key.clone(), old),
            None => self.tree.remove_property_internal(&self.key),
        }
        self.tree.fire_property_changed(&self.key);
        true
    }
    fn get_description(&self) -> Option<&str> {
        Some("set property")
    }
}

/// Recorded by [`ValueTree::remove_property_with`] and [`ValueTree::remove_property`].
pub struct RemovePropertyAction {
    pub(crate) tree: ValueTree,
    pub(crate) key: Identifier,
    pub(crate) old_value: Value,
}

impl UndoableAction for RemovePropertyAction {
    fn perform(&mut self, _manager: &UndoManager) -> bool {
        self.tree.remove_property_internal(&self.key);
        self.tree.fire_property_changed(&self.key);
        true
    }
    fn undo(&mut self, _manager: &UndoManager) -> bool {
        self.tree
            .set_property_internal(self.key.clone(), self.old_value.clone());
        self.tree.fire_property_changed(&self.key);
        true
    }
    fn get_description(&self) -> Option<&str> {
        Some("remove property")
    }
}

/// Recorded by [`ValueTree::add_child_with`] and [`ValueTree::add_child`].
pub struct AddChildAction {
    pub(crate) parent: ValueTree,
    pub(crate) child: ValueTree,
    pub(crate) index: usize,
}

impl UndoableAction for AddChildAction {
    fn perform(&mut self, _manager: &UndoManager) -> bool {
        self.parent.add_child_internal(self.child.clone(), self.index);
        self.parent.fire_child_added(&self.child, self.index);
        true
    }
    fn undo(&mut self, _manager: &UndoManager) -> bool {
        if let Some(idx) = self.parent.remove_child_internal(&self.child) {
            self.parent.fire_child_removed(&self.child, idx);
        }
        true
    }
    fn get_description(&self) -> Option<&str> {
        Some("add child")
    }
}

/// Recorded by [`ValueTree::remove_child_with`] and [`ValueTree::remove_child`].
pub struct RemoveChildAction {
    pub(crate) parent: ValueTree,
    pub(crate) child: ValueTree,
    pub(crate) index: usize,
}

impl UndoableAction for RemoveChildAction {
    fn perform(&mut self, _manager: &UndoManager) -> bool {
        if let Some(idx) = self.parent.remove_child_internal(&self.child) {
            self.parent.fire_child_removed(&self.child, idx);
        }
        true
    }
    fn undo(&mut self, _manager: &UndoManager) -> bool {
        self.parent.add_child_internal(self.child.clone(), self.index);
        self.parent.fire_child_added(&self.child, self.index);
        true
    }
    fn get_description(&self) -> Option<&str> {
        Some("remove child")
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::Identifier;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn perform_and_undo_round_trip() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        tree.set_property_with("gain", 0.5_f64, &manager);
        assert_eq!(tree.get_double(&Identifier::new("gain"), 0.0), 0.5);
        assert!(manager.can_undo());
        assert!(manager.undo());
        assert!(!tree.has_property(&Identifier::new("gain")));
        assert!(manager.can_redo());
        assert!(manager.redo());
        assert_eq!(tree.get_double(&Identifier::new("gain"), 0.0), 0.5);
    }

    #[test]
    fn undo_restores_previous_property_value() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        tree.set_property_with("k", 1_i64, &manager);
        tree.set_property_with("k", 2_i64, &manager);
        tree.set_property_with("k", 3_i64, &manager);
        assert_eq!(tree.get_int(&Identifier::new("k"), 0), 3);
        manager.undo();
        assert_eq!(tree.get_int(&Identifier::new("k"), 0), 2);
        manager.undo();
        assert_eq!(tree.get_int(&Identifier::new("k"), 0), 1);
        manager.undo();
        assert!(!tree.has_property(&Identifier::new("k")));
    }

    #[test]
    fn attached_undo_manager_records_implicitly() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        tree.set_undo_manager(Some(&manager));
        tree.set_property("gain", 0.25_f64);
        assert!(manager.can_undo());
        manager.undo();
        assert!(!tree.has_property(&Identifier::new("gain")));
        // Detaching prevents further recording
        tree.set_undo_manager(None);
        tree.set_property("gain", 1.0_f64);
        assert_eq!(manager.redo_count(), 1); // first action moved to redo on undo()
    }

    #[test]
    fn child_add_remove_undo() {
        let parent = ValueTree::new("Parent");
        let manager = UndoManager::new();
        let child = ValueTree::new("Child");
        parent.add_child_with(child.clone(), 0, &manager);
        assert_eq!(parent.num_children(), 1);
        manager.undo();
        assert_eq!(parent.num_children(), 0);
        manager.redo();
        assert_eq!(parent.num_children(), 1);
    }

    #[test]
    fn transaction_groups_actions() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        manager.begin_transaction("init");
        tree.set_property_with("a", 1_i64, &manager);
        tree.set_property_with("b", 2_i64, &manager);
        tree.set_property_with("c", 3_i64, &manager);
        manager.end_transaction();
        assert_eq!(manager.undo_count(), 1);
        manager.undo();
        assert!(!tree.has_property(&Identifier::new("a")));
        assert!(!tree.has_property(&Identifier::new("b")));
        assert!(!tree.has_property(&Identifier::new("c")));
        // The transaction is now on the redo stack.
        assert_eq!(manager.redo_description().as_deref(), Some("init"));
    }

    #[test]
    fn empty_transaction_is_noop() {
        let manager = UndoManager::new();
        manager.begin_transaction("nothing");
        assert!(!manager.end_transaction());
        assert_eq!(manager.undo_count(), 0);
    }

    #[test]
    fn cancel_transaction_discards_actions() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        manager.begin_transaction("scratch");
        tree.set_property_with("x", 1_i64, &manager);
        assert!(manager.cancel_transaction());
        assert_eq!(manager.undo_count(), 0);
        // The mutation is still on the tree (it was performed).
        assert!(tree.has_property(&Identifier::new("x")));
    }

    #[test]
    fn nested_transactions() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        manager.begin_transaction("outer");
        tree.set_property_with("a", 1_i64, &manager);
        manager.begin_transaction("inner");
        tree.set_property_with("b", 2_i64, &manager);
        manager.end_transaction();
        assert!(manager.is_in_transaction());
        manager.end_transaction();
        assert!(!manager.is_in_transaction());
        assert_eq!(manager.undo_count(), 2);
    }

    #[test]
    fn storage_limit_drops_oldest() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        manager.set_max_undo_storage_size(3);
        for i in 0..10 {
            tree.set_property_with("k", i, &manager);
        }
        assert!(manager.can_undo());
        // Storage limit should have trimmed the oldest actions.
        let count = manager.undo_count();
        assert!(count <= 3, "expected at most 3 actions, got {count}");
    }

    #[test]
    fn clear_history_drops_everything() {
        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        tree.set_property_with("a", 1_i64, &manager);
        tree.set_property_with("b", 2_i64, &manager);
        manager.clear_history();
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn clone_shares_state() {
        let manager = UndoManager::new();
        let manager2 = manager.clone();
        let tree = ValueTree::new("Root");
        tree.set_property_with("a", 1_i64, &manager);
        assert!(manager2.can_undo());
    }

    static SERIAL: AtomicUsize = AtomicUsize::new(0);

    fn next_serial() -> usize {
        SERIAL.fetch_add(1, Ordering::SeqCst)
    }

    #[test]
    fn custom_action_round_trip() {
        struct CounterAction {
            tree: ValueTree,
            key: Identifier,
            increment: i64,
        }
        impl UndoableAction for CounterAction {
            fn perform(&mut self, _manager: &UndoManager) -> bool {
                let cur = self.tree.get_int(&self.key, 0);
                self.tree
                    .set_property_internal(self.key.clone(), Value::Int(cur + self.increment));
                self.tree.fire_property_changed(&self.key);
                true
            }
            fn undo(&mut self, _manager: &UndoManager) -> bool {
                let cur = self.tree.get_int(&self.key, 0);
                self.tree
                    .set_property_internal(self.key.clone(), Value::Int(cur - self.increment));
                self.tree.fire_property_changed(&self.key);
                true
            }
            fn get_description(&self) -> Option<&str> {
                Some("counter")
            }
        }

        let tree = ValueTree::new("Root");
        let manager = UndoManager::new();
        let key = Identifier::new("count");
        manager.perform(CounterAction {
            tree: tree.clone(),
            key: key.clone(),
            increment: 5,
        });
        assert_eq!(tree.get_int(&key, 0), 5);
        manager.perform(CounterAction {
            tree: tree.clone(),
            key: key.clone(),
            increment: 3,
        });
        assert_eq!(tree.get_int(&key, 0), 8);
        manager.undo();
        assert_eq!(tree.get_int(&key, 0), 5);
        manager.undo();
        assert_eq!(tree.get_int(&key, 0), 0);
    }

    #[test]
    fn no_undo_when_empty() {
        let manager = UndoManager::new();
        assert!(!manager.can_undo());
        assert!(!manager.undo());
        assert!(!manager.can_redo());
        assert!(!manager.redo());
    }

    #[test]
    fn ensure_serial_helper_works() {
        // Quiet the unused warning without a real test.
        assert!(next_serial() < usize::MAX);
    }
}
