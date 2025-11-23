//! UndoManager implementation for tracking and reverting state changes.
//!
//! This module provides an undo/redo system that allows tracking state changes
//! and reverting them. Actions are represented by the `UndoableAction` trait.
//!
//! # Examples
//!
//! ```
//! use nih_plug_data::{UndoManager, UndoableAction};
//! use std::sync::{Arc, Mutex};
//!
//! struct SetValueAction {
//!     value: i32,
//!     old_value: i32,
//!     target: Arc<Mutex<i32>>,
//! }
//!
//! impl UndoableAction for SetValueAction {
//!     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//!         *self.target.lock().unwrap() = self.value;
//!         Ok(())
//!     }
//!
//!     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//!         *self.target.lock().unwrap() = self.old_value;
//!         Ok(())
//!     }
//! }
//!
//! let mut manager = UndoManager::new();
//! let target = Arc::new(Mutex::new(0));
//!
//! let action = Box::new(SetValueAction {
//!     value: 42,
//!     old_value: 0,
//!     target: target.clone(),
//! });
//!
//! manager.perform(action).unwrap();
//! assert_eq!(*target.lock().unwrap(), 42);
//!
//! manager.undo().unwrap();
//! assert_eq!(*target.lock().unwrap(), 0);
//! ```

use crate::error::DataError;

/// A trait for actions that can be undone and redone.
///
/// Implementors of this trait represent reversible operations that can be
/// tracked by the `UndoManager`.
pub trait UndoableAction: Send {
    /// Performs the action.
    ///
    /// This method is called when the action is first executed or when it's redone.
    ///
    /// # Errors
    ///
    /// Returns an error if the action cannot be performed.
    fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Undoes the action.
    ///
    /// This method should reverse the effects of `perform()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the action cannot be undone.
    fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Manages undo/redo functionality for a sequence of actions.
///
/// The `UndoManager` maintains two stacks: one for undo operations and one for redo operations.
/// When an action is performed, it's added to the undo stack. When an action is undone, it's
/// moved to the redo stack. Performing a new action clears the redo stack.
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance.
///
/// # Examples
///
/// ```
/// use nih_plug_data::UndoManager;
///
/// let mut manager = UndoManager::new();
/// assert!(!manager.can_undo());
/// assert!(!manager.can_redo());
/// ```
pub struct UndoManager {
    undo_stack: Vec<Box<dyn UndoableAction>>,
    redo_stack: Vec<Box<dyn UndoableAction>>,
    transaction_stack: Vec<Vec<Box<dyn UndoableAction>>>,
}

impl UndoManager {
    /// Creates a new empty `UndoManager`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::UndoManager;
    ///
    /// let manager = UndoManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            transaction_stack: Vec::new(),
        }
    }

    /// Performs an action and adds it to the undo stack.
    ///
    /// This clears the redo stack, as performing a new action invalidates
    /// any previously undone actions.
    ///
    /// # Errors
    ///
    /// Returns an error if the action's `perform()` method fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{UndoManager, UndoableAction};
    ///
    /// struct DummyAction;
    ///
    /// impl UndoableAction for DummyAction {
    ///     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    ///
    ///     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let mut manager = UndoManager::new();
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// assert!(manager.can_undo());
    /// ```
    pub fn perform(&mut self, mut action: Box<dyn UndoableAction>) -> Result<(), DataError> {
        action
            .perform()
            .map_err(|e| DataError::ActionFailed(e.to_string()))?;

        // If we're in a transaction, add to the transaction stack
        if let Some(transaction) = self.transaction_stack.last_mut() {
            transaction.push(action);
        } else {
            // Clear redo stack when performing a new action
            self.redo_stack.clear();
            self.undo_stack.push(action);
        }

        Ok(())
    }

    /// Undoes the most recent action.
    ///
    /// The action is moved from the undo stack to the redo stack.
    ///
    /// # Errors
    ///
    /// Returns an error if there are no actions to undo or if the action's
    /// `undo()` method fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{UndoManager, UndoableAction};
    ///
    /// struct DummyAction;
    ///
    /// impl UndoableAction for DummyAction {
    ///     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    ///
    ///     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let mut manager = UndoManager::new();
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// manager.undo().unwrap();
    /// assert!(!manager.can_undo());
    /// assert!(manager.can_redo());
    /// ```
    pub fn undo(&mut self) -> Result<(), DataError> {
        let mut action = self
            .undo_stack
            .pop()
            .ok_or(DataError::NoActionsToUndo)?;

        action
            .undo()
            .map_err(|e| DataError::ActionFailed(e.to_string()))?;

        self.redo_stack.push(action);
        Ok(())
    }

    /// Redoes the most recently undone action.
    ///
    /// The action is moved from the redo stack back to the undo stack.
    ///
    /// # Errors
    ///
    /// Returns an error if there are no actions to redo or if the action's
    /// `perform()` method fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{UndoManager, UndoableAction};
    ///
    /// struct DummyAction;
    ///
    /// impl UndoableAction for DummyAction {
    ///     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    ///
    ///     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let mut manager = UndoManager::new();
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// manager.undo().unwrap();
    /// manager.redo().unwrap();
    /// assert!(manager.can_undo());
    /// assert!(!manager.can_redo());
    /// ```
    pub fn redo(&mut self) -> Result<(), DataError> {
        let mut action = self
            .redo_stack
            .pop()
            .ok_or(DataError::NoActionsToRedo)?;

        action
            .perform()
            .map_err(|e| DataError::ActionFailed(e.to_string()))?;

        self.undo_stack.push(action);
        Ok(())
    }

    /// Returns `true` if there are actions that can be undone.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{UndoManager, UndoableAction};
    ///
    /// struct DummyAction;
    ///
    /// impl UndoableAction for DummyAction {
    ///     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    ///
    ///     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let mut manager = UndoManager::new();
    /// assert!(!manager.can_undo());
    ///
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// assert!(manager.can_undo());
    /// ```
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there are actions that can be redone.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{UndoManager, UndoableAction};
    ///
    /// struct DummyAction;
    ///
    /// impl UndoableAction for DummyAction {
    ///     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    ///
    ///     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let mut manager = UndoManager::new();
    /// assert!(!manager.can_redo());
    ///
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// manager.undo().unwrap();
    /// assert!(manager.can_redo());
    /// ```
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Begins a transaction.
    ///
    /// Actions performed within a transaction are grouped together and can be
    /// undone/redone as a single unit. Transactions can be nested.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{UndoManager, UndoableAction};
    ///
    /// struct DummyAction;
    ///
    /// impl UndoableAction for DummyAction {
    ///     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    ///
    ///     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let mut manager = UndoManager::new();
    /// manager.begin_transaction();
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// manager.end_transaction();
    ///
    /// // Both actions are undone together
    /// manager.undo().unwrap();
    /// assert!(!manager.can_undo());
    /// ```
    pub fn begin_transaction(&mut self) {
        self.transaction_stack.push(Vec::new());
    }

    /// Ends the current transaction.
    ///
    /// All actions performed since the matching `begin_transaction()` call
    /// are grouped into a single composite action.
    ///
    /// # Panics
    ///
    /// Panics if there is no active transaction.
    pub fn end_transaction(&mut self) {
        if let Some(actions) = self.transaction_stack.pop() {
            if !actions.is_empty() {
                // Clear redo stack when ending a transaction
                self.redo_stack.clear();
                
                // Create a composite action
                let composite = Box::new(CompositeAction { actions });
                self.undo_stack.push(composite);
            }
        } else {
            panic!("end_transaction called without matching begin_transaction");
        }
    }

    /// Clears all undo and redo history.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{UndoManager, UndoableAction};
    ///
    /// struct DummyAction;
    ///
    /// impl UndoableAction for DummyAction {
    ///     fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    ///
    ///     fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let mut manager = UndoManager::new();
    /// manager.perform(Box::new(DummyAction)).unwrap();
    /// manager.clear();
    /// assert!(!manager.can_undo());
    /// ```
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.transaction_stack.clear();
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A composite action that groups multiple actions together.
///
/// This is used internally by the transaction system to group actions
/// that should be undone/redone as a single unit.
struct CompositeAction {
    actions: Vec<Box<dyn UndoableAction>>,
}

impl UndoableAction for CompositeAction {
    fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for action in &mut self.actions {
            action.perform()?;
        }
        Ok(())
    }

    fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Undo in reverse order
        for action in self.actions.iter_mut().rev() {
            action.undo()?;
        }
        Ok(())
    }
}
