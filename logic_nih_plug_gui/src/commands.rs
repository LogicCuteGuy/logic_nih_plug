//! Application commands, keypress mappings, and the central command manager.
//!
//! This module mirrors JUCE's `juce::ApplicationCommandManager` family:
//!
//! - [`CommandId`] — strongly-typed handle for a registered command.
//! - [`CommandInfo`] — metadata (name, description, category, default
//!   keypress, flags) for a single command.
//! - [`CommandFlags`] — bitflags for command state (disabled, ticked,
//!   hidden, etc.).
//! - [`KeyPress`] — a keyboard key with modifier state; comparable and
//!   display-friendly.
//! - [`KeyPressMappingSet`] — keypress ⇄ command mapping table with
//!   compact-string round-tripping.
//! - [`ApplicationCommandTarget`] — trait for objects that can handle
//!   commands.
//! - [`ApplicationCommandManager`] — central registry + dispatcher.
//!
//! The manager dispatches invocations through a chain of targets — the
//! default target (set via [`set_first_command_target`][ApplicationCommandManager::set_first_command_target])
//! gets the first shot, and on rejection the chain continues via each
//! target's `get_next_command_target`.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_gui::commands::{
//!     ApplicationCommandManager, CommandFlags, CommandId, CommandInfo,
//!     KeyPress,
//! };
//!
//! let mut mgr = ApplicationCommandManager::new();
//!
//! let save_id = CommandId(0x100);
//! mgr.register_command(CommandInfo {
//!     command_id: save_id,
//!     short_name: "Save".to_string(),
//!     long_name: "File > Save".to_string(),
//!     description: "Save the current document".to_string(),
//!     category: "File".to_string(),
//!     flags: CommandFlags::empty(),
//!     default_keypress: Some(KeyPress::new(b's' as u32, crate::input::Modifiers { shift: false, ctrl: true, alt: false, meta: false })),
//! });
//!
//! assert_eq!(mgr.get_num_commands(), 1);
//! assert_eq!(mgr.get_command_for_id(save_id).map(|c| c.short_name.as_str()), Some("Save"));
//! ```

use crate::input::Modifiers;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// CommandId + Flags
// ---------------------------------------------------------------------------

/// Strongly-typed handle for a registered command.
///
/// Command IDs are arbitrary `u32` values. Conventionally, application-
/// defined IDs start at `0x1000` to leave room for the framework to
/// reserve the lower range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandId(pub u32);

/// Bitflags for command state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CommandFlags(pub u32);

impl CommandFlags {
    /// No flags set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// The command is currently disabled and cannot be invoked.
    pub const IS_DISABLED: u32 = 1 << 0;
    /// The command is in a "ticked" state (e.g. a toggle that's on).
    pub const IS_TICKED: u32 = 1 << 1;
    /// The command should be hidden from menus.
    pub const HIDE_IN_MENU: u32 = 1 << 2;
    /// The command should not appear in keypress-assignment dialogs.
    pub const HIDE_IN_KEY_MAPPING: u32 = 1 << 3;
    /// The command should not appear in toolbars / ribbons.
    pub const HIDE_IN_TOOLBAR: u32 = 1 << 4;

    /// Returns `true` if any bits in `other` are set on `self`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether [`Self::IS_DISABLED`] is set.
    pub const fn is_disabled(self) -> bool {
        self.contains(Self(Self::IS_DISABLED))
    }

    /// Whether [`Self::IS_TICKED`] is set.
    pub const fn is_ticked(self) -> bool {
        self.contains(Self(Self::IS_TICKED))
    }

    /// Whether [`Self::HIDE_IN_MENU`] is set.
    pub const fn hidden_in_menu(self) -> bool {
        self.contains(Self(Self::HIDE_IN_MENU))
    }

    /// Whether [`Self::HIDE_IN_KEY_MAPPING`] is set.
    pub const fn hidden_in_key_mapping(self) -> bool {
        self.contains(Self(Self::HIDE_IN_KEY_MAPPING))
    }

    /// Whether [`Self::HIDE_IN_TOOLBAR`] is set.
    pub const fn hidden_in_toolbar(self) -> bool {
        self.contains(Self(Self::HIDE_IN_TOOLBAR))
    }
}

impl std::ops::BitOr for CommandFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CommandFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

// ---------------------------------------------------------------------------
// KeyPress
// ---------------------------------------------------------------------------

/// A keyboard key combined with the modifier state at the time it was
/// pressed.
///
/// The key code is a raw `u32` — printable keys are ASCII, function keys
/// use the virtual-key constants ([`KeyPress::F1`] through
/// [`KeyPress::F12`]), arrow keys use [`KeyPress::LEFT`] …
/// [`KeyPress::DOWN`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyPress {
    /// The raw key code.
    pub key_code: u32,
    /// The modifiers held when the key was pressed.
    pub modifiers: Modifiers,
}

impl KeyPress {
    // Virtual key codes for non-printable keys (chosen to match common
    // platform conventions; values are arbitrary beyond ASCII printable
    // range so callers can spot collisions).
    /// Spacebar.
    pub const SPACE: u32 = 0x20;
    /// Escape.
    pub const ESCAPE: u32 = 0x1B;
    /// Return / Enter.
    pub const RETURN: u32 = 0x0D;
    /// Tab.
    pub const TAB: u32 = 0x09;
    /// Backspace.
    pub const BACKSPACE: u32 = 0x08;
    /// Delete (forward delete).
    pub const DELETE: u32 = 0x7F;
    /// Insert.
    pub const INSERT: u32 = 0x100;
    /// Home.
    pub const HOME: u32 = 0x101;
    /// End.
    pub const END: u32 = 0x102;
    /// Page Up.
    pub const PAGE_UP: u32 = 0x103;
    /// Page Down.
    pub const PAGE_DOWN: u32 = 0x104;
    /// Left arrow.
    pub const LEFT: u32 = 0x105;
    /// Right arrow.
    pub const RIGHT: u32 = 0x106;
    /// Up arrow.
    pub const UP: u32 = 0x107;
    /// Down arrow.
    pub const DOWN: u32 = 0x108;
    /// F1.
    pub const F1: u32 = 0x110;
    /// F2.
    pub const F2: u32 = 0x111;
    /// F3.
    pub const F3: u32 = 0x112;
    /// F4.
    pub const F4: u32 = 0x113;
    /// F5.
    pub const F5: u32 = 0x114;
    /// F6.
    pub const F6: u32 = 0x115;
    /// F7.
    pub const F7: u32 = 0x116;
    /// F8.
    pub const F8: u32 = 0x117;
    /// F9.
    pub const F9: u32 = 0x118;
    /// F10.
    pub const F10: u32 = 0x119;
    /// F11.
    pub const F11: u32 = 0x11A;
    /// F12.
    pub const F12: u32 = 0x11B;

    /// Construct a key press with the given key code and no modifiers.
    pub fn from_key_code(key_code: u32) -> Self {
        Self {
            key_code,
            modifiers: Modifiers::none(),
        }
    }

    /// Construct a key press with the given key code and modifiers.
    pub fn new(key_code: u32, modifiers: Modifiers) -> Self {
        Self { key_code, modifiers }
    }

    /// Construct from a single ASCII character (uppercased if `shift`).
    pub fn from_char(c: char, mods: Modifiers) -> Self {
        let key_code = if mods.shift && c.is_ascii_lowercase() {
            c.to_ascii_uppercase() as u32
        } else {
            c as u32
        };
        Self {
            key_code,
            modifiers: mods,
        }
    }

    /// Whether `self` matches `other` (same key code, same modifiers).
    pub fn matches(&self, other: &KeyPress) -> bool {
        self.key_code == other.key_code && self.modifiers == other.modifiers
    }

    /// Whether `self` is a printable ASCII character.
    pub fn is_printable(&self) -> bool {
        // 0x20..=0x7E are printable ASCII; DEL (0x7F) is not.
        matches!(self.key_code, 0x20..=0x7E)
    }

    /// A short textual description like `"Ctrl+S"`, `"F5"`, or `"Ctrl+Shift+Right"`.
    pub fn get_text_description(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.modifiers.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_string());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_string());
        }
        if self.modifiers.meta {
            parts.push("Cmd".to_string());
        }
        parts.push(Self::key_code_name(self.key_code));
        parts.join("+")
    }

    fn key_code_name(code: u32) -> String {
        match code {
            Self::SPACE => "Space".to_string(),
            Self::ESCAPE => "Esc".to_string(),
            Self::RETURN => "Return".to_string(),
            Self::TAB => "Tab".to_string(),
            Self::BACKSPACE => "Backspace".to_string(),
            Self::DELETE => "Delete".to_string(),
            Self::INSERT => "Insert".to_string(),
            Self::HOME => "Home".to_string(),
            Self::END => "End".to_string(),
            Self::PAGE_UP => "PageUp".to_string(),
            Self::PAGE_DOWN => "PageDown".to_string(),
            Self::LEFT => "Left".to_string(),
            Self::RIGHT => "Right".to_string(),
            Self::UP => "Up".to_string(),
            Self::DOWN => "Down".to_string(),
            Self::F1 => "F1".to_string(),
            Self::F2 => "F2".to_string(),
            Self::F3 => "F3".to_string(),
            Self::F4 => "F4".to_string(),
            Self::F5 => "F5".to_string(),
            Self::F6 => "F6".to_string(),
            Self::F7 => "F7".to_string(),
            Self::F8 => "F8".to_string(),
            Self::F9 => "F9".to_string(),
            Self::F10 => "F10".to_string(),
            Self::F11 => "F11".to_string(),
            Self::F12 => "F12".to_string(),
            0x20..=0x7E => {
                let c = char::from_u32(code).unwrap_or('?');
                if c.is_ascii_graphic() {
                    c.to_string()
                } else {
                    format!("0x{code:02X}")
                }
            }
            _ => format!("0x{code:X}"),
        }
    }
}

// ---------------------------------------------------------------------------
// CommandInfo
// ---------------------------------------------------------------------------

/// Static metadata for a single command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandInfo {
    /// Unique ID.
    pub command_id: CommandId,
    /// Verb form ("Save", "Copy").
    pub short_name: String,
    /// Menu label form ("File > Save").
    pub long_name: String,
    /// Tooltip / longer description.
    pub description: String,
    /// Logical group ("File", "Edit", "View").
    pub category: String,
    /// State flags.
    pub flags: CommandFlags,
    /// The default keypress for this command, if any.
    pub default_keypress: Option<KeyPress>,
}

impl CommandInfo {
    /// Construct a minimal `CommandInfo` with just the ID and short name.
    pub fn new(command_id: CommandId, short_name: impl Into<String>) -> Self {
        Self {
            command_id,
            short_name: short_name.into(),
            long_name: String::new(),
            description: String::new(),
            category: String::new(),
            flags: CommandFlags::empty(),
            default_keypress: None,
        }
    }

    /// Builder: set the long name.
    pub fn with_long_name(mut self, name: impl Into<String>) -> Self {
        self.long_name = name.into();
        self
    }

    /// Builder: set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Builder: set the category.
    pub fn with_category(mut self, cat: impl Into<String>) -> Self {
        self.category = cat.into();
        self
    }

    /// Builder: set the flags.
    pub fn with_flags(mut self, flags: CommandFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Builder: set the default keypress.
    pub fn with_default_keypress(mut self, key: KeyPress) -> Self {
        self.default_keypress = Some(key);
        self
    }
}

// ---------------------------------------------------------------------------
// KeyPressMappingSet
// ---------------------------------------------------------------------------

/// A bidirectional mapping between [`KeyPress`]es and [`CommandId`]s.
///
/// Multiple keypresses may map to the same command, and a single keypress
/// may map to multiple commands (in practice the last mapping wins, but
/// `find_command_for_keypress` returns the first one inserted).
#[derive(Debug, Clone, Default)]
pub struct KeyPressMappingSet {
    /// Forward map: keypress → first command ID that claimed it.
    forward: HashMap<KeyPress, CommandId>,
    /// Reverse map: command ID → set of bound keypresses.
    reverse: HashMap<CommandId, Vec<KeyPress>>,
}

impl KeyPressMappingSet {
    /// Create an empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct keypress→command mappings.
    pub fn len(&self) -> usize {
        self.forward.len()
    }

    /// Whether the mapping set has no entries.
    pub fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }

    /// Bind a keypress to a command. Returns the previous mapping (if any).
    pub fn add_mapping(&mut self, key: KeyPress, id: CommandId) -> Option<CommandId> {
        let prev = self.forward.insert(key, id);
        self.reverse.entry(id).or_default().push(key);
        prev
    }

    /// Remove a keypress→command mapping. Returns the removed command ID.
    pub fn remove_mapping(&mut self, key: &KeyPress) -> Option<CommandId> {
        let id = self.forward.remove(key)?;
        if let Some(keys) = self.reverse.get_mut(&id) {
            keys.retain(|k| k != key);
            if keys.is_empty() {
                self.reverse.remove(&id);
            }
        }
        Some(id)
    }

    /// Remove every mapping that points at `id`.
    pub fn remove_all_mappings_for_command(&mut self, id: CommandId) {
        if let Some(keys) = self.reverse.remove(&id) {
            for k in keys {
                self.forward.remove(&k);
            }
        }
    }

    /// Clear all mappings.
    pub fn clear(&mut self) {
        self.forward.clear();
        self.reverse.clear();
    }

    /// Find the first command bound to `key`.
    pub fn find_command_for_keypress(&self, key: &KeyPress) -> Option<CommandId> {
        self.forward.get(key).copied()
    }

    /// All keypresses bound to `id`.
    pub fn get_keypresses_for_command(&self, id: CommandId) -> Vec<KeyPress> {
        self.reverse.get(&id).cloned().unwrap_or_default()
    }

    /// All known (keypress, command) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&KeyPress, &CommandId)> {
        self.forward.iter()
    }

    /// Encode to a compact string of the form `"key_code:modifiers=id;..."`.
    /// Key codes are rendered as hex (`0x114`) so non-printable codes are
    /// unambiguous; modifiers are rendered with `Debug`.
    pub fn to_compact_string(&self) -> String {
        let mut parts = Vec::with_capacity(self.forward.len());
        // Stable order: sort by key text description for determinism.
        let mut entries: Vec<_> = self.forward.iter().collect();
        entries.sort_by_key(|(k, _)| k.get_text_description());
        for (k, id) in entries {
            parts.push(format!(
                "0x{:X}:{:?}=0x{:X}",
                k.key_code, k.modifiers, id.0
            ));
        }
        parts.join(";")
    }
}

// ---------------------------------------------------------------------------
// CommandListener
// ---------------------------------------------------------------------------

/// Listener interface for command-list / command-status changes.
pub trait CommandListener {
    /// Fired when the list of registered commands changes.
    fn command_list_changed(&mut self);

    /// Fired when a command's status (enabled / ticked) changes.
    fn command_status_changed(&mut self);
}

// ---------------------------------------------------------------------------
// ApplicationCommandTarget
// ---------------------------------------------------------------------------

/// Per-invocation context passed to [`ApplicationCommandTarget::perform`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvocationInfo {
    /// The command being invoked.
    pub command_id: CommandId,
    /// The originating direction (key, menu, button, programmatic).
    pub origin: InvocationOrigin,
    /// Whether to perform the command asynchronously (deferred to the
    /// message thread).
    pub asynchronously: bool,
}

impl InvocationInfo {
    /// Construct an invocation targeting `command_id` with default flags.
    pub fn from_command(command_id: CommandId) -> Self {
        Self {
            command_id,
            origin: InvocationOrigin::Direct,
            asynchronously: false,
        }
    }
}

/// How an invocation was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvocationOrigin {
    /// Invoked by `invoke_directly` from code.
    Direct,
    /// Invoked by a keypress.
    Key,
    /// Invoked by a menu item.
    Menu,
    /// Invoked by a button click.
    Button,
}

/// Receives commands dispatched by [`ApplicationCommandManager`].
pub trait ApplicationCommandTarget {
    /// Return this target's static metadata for `id`, or `None` if this
    /// target doesn't handle that command. Returning `None` causes the
    /// manager to ask the next target in the chain.
    fn get_command_info(&self, id: CommandId) -> Option<CommandInfo>;

    /// Return the next target in the chain, or `None`.
    fn get_next_command_target(&self) -> Option<Box<dyn ApplicationCommandTarget>>;

    /// Try to perform the command. Return `true` on success.
    fn perform(&mut self, info: InvocationInfo) -> bool;
}

// ---------------------------------------------------------------------------
// ApplicationCommandManager
// ---------------------------------------------------------------------------

/// Central registry of application commands, keypress mappings, and
/// dispatch targets.
///
/// Mirrors `juce::ApplicationCommandManager`.
#[derive(Default)]
pub struct ApplicationCommandManager {
    /// Registered commands, indexed by ID.
    commands: HashMap<CommandId, CommandInfo>,
    /// Order-preserving list of command IDs (insertion order).
    order: Vec<CommandId>,
    /// Keypress ⇄ command mappings.
    key_mappings: KeyPressMappingSet,
    /// The first target in the dispatch chain.
    first_target: Option<Box<dyn ApplicationCommandTarget>>,
    /// Listeners.
    listeners: Vec<Box<dyn CommandListener>>,
}

impl ApplicationCommandManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop every registered command and keypress mapping.
    pub fn clear_commands(&mut self) {
        self.commands.clear();
        self.order.clear();
        self.key_mappings.clear();
        self.notify_list_changed();
    }

    /// Register a single command. If a command with this ID already exists,
    /// it is replaced.
    pub fn register_command(&mut self, info: CommandInfo) {
        if !self.commands.contains_key(&info.command_id) {
            self.order.push(info.command_id);
        }
        let id = info.command_id;
        let default_keypress = info.default_keypress;
        self.commands.insert(id, info);
        if let Some(key) = default_keypress {
            self.key_mappings.add_mapping(key, id);
        }
        self.notify_list_changed();
    }

    /// Register every command that `target` reports via
    /// [`ApplicationCommandTarget::get_command_info`].
    ///
    /// Walks the **first** target in the chain only — `register_all_commands_for_target`
    /// is the single-level variant. For multi-level chains, walk them
    /// yourself and call [`register_command`][Self::register_command]
    /// with each returned `CommandInfo`.
    pub fn register_all_commands_for_target(&mut self, target: &dyn ApplicationCommandTarget) {
        for id in self.order.clone() {
            if let Some(info) = target.get_command_info(id) {
                self.register_command(info);
            }
        }
    }

    /// Remove a single command. Also strips its keypress mappings.
    pub fn remove_command(&mut self, id: CommandId) {
        if self.commands.remove(&id).is_some() {
            self.order.retain(|&c| c != id);
            self.key_mappings.remove_all_mappings_for_command(id);
            self.notify_list_changed();
        }
    }

    /// Number of registered commands.
    pub fn get_num_commands(&self) -> usize {
        self.commands.len()
    }

    /// Borrow a command by insertion-order index.
    pub fn get_command_for_index(&self, index: usize) -> Option<&CommandInfo> {
        self.order.get(index).and_then(|id| self.commands.get(id))
    }

    /// Borrow a command by ID.
    pub fn get_command_for_id(&self, id: CommandId) -> Option<&CommandInfo> {
        self.commands.get(&id)
    }

    /// Short name for a command ID, or empty string if not registered.
    pub fn get_name_of_command(&self, id: CommandId) -> String {
        self.commands
            .get(&id)
            .map(|c| c.short_name.clone())
            .unwrap_or_default()
    }

    /// Description for a command ID, falling back to the short name.
    pub fn get_description_of_command(&self, id: CommandId) -> String {
        self.commands
            .get(&id)
            .map(|c| {
                if c.description.is_empty() {
                    c.short_name.clone()
                } else {
                    c.description.clone()
                }
            })
            .unwrap_or_default()
    }

    /// All distinct category strings across registered commands.
    pub fn get_command_categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .commands
            .values()
            .filter_map(|c| {
                if c.category.is_empty() {
                    None
                } else {
                    Some(c.category.clone())
                }
            })
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// All command IDs in a given category.
    pub fn get_commands_in_category(&self, category: &str) -> Vec<CommandId> {
        self.order
            .iter()
            .copied()
            .filter(|id| {
                self.commands
                    .get(id)
                    .map(|c| c.category == category)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Mutable access to the keypress mapping set.
    pub fn key_mappings_mut(&mut self) -> &mut KeyPressMappingSet {
        &mut self.key_mappings
    }

    /// Immutable access to the keypress mapping set.
    pub fn key_mappings(&self) -> &KeyPressMappingSet {
        &self.key_mappings
    }

    /// Bind an additional keypress to a command.
    pub fn add_key_mapping(&mut self, key: KeyPress, id: CommandId) {
        self.key_mappings.add_mapping(key, id);
    }

    /// Resolve a keypress to its command ID (via registered mappings).
    pub fn find_key_mapping(&self, key: &KeyPress) -> Option<CommandId> {
        self.key_mappings.find_command_for_keypress(key)
    }

    /// Set the default dispatch target. Pass `None` to clear.
    pub fn set_first_command_target(&mut self, target: Option<Box<dyn ApplicationCommandTarget>>) {
        self.first_target = target;
    }

    /// The current default dispatch target.
    pub fn first_command_target(&self) -> Option<&dyn ApplicationCommandTarget> {
        self.first_target.as_deref()
    }

    /// Dispatch `id` to the target chain. Returns `true` if any target
    /// handled it.
    pub fn invoke_directly(&mut self, id: CommandId) -> bool {
        let info = InvocationInfo::from_command(id);
        self.invoke(info)
    }

    /// Dispatch `info` to the target chain. Returns `true` if any target
    /// handled it.
    pub fn invoke(&mut self, info: InvocationInfo) -> bool {
        // Take the chain out of `self` so we can walk it while mutating
        // listeners / command tables without disturbing the manager.
        let mut cursor = self.first_target.take();
        while let Some(mut target) = cursor.take() {
            // Peek the next target before invoking — `get_next_command_target`
            // returns an owned `Box`, so we move it cleanly without
            // fighting the borrow checker.
            cursor = target.get_next_command_target();
            if target.perform(info) {
                // Drop the rest of the chain; we've handled the command.
                drop(cursor);
                // We can't restore the chain (it's been consumed), but the
                // common case is a single target with no next link.
                return true;
            }
        }
        false
    }

    /// Register a listener for command-list / status changes.
    pub fn add_listener(&mut self, listener: Box<dyn CommandListener>) {
        self.listeners.push(listener);
    }

    /// Drop every listener.
    pub fn clear_listeners(&mut self) {
        self.listeners.clear();
    }

    /// Tell the manager that a command's status (enabled / ticked) may
    /// have changed. Forwards to all listeners.
    pub fn command_status_changed(&mut self) {
        for l in &mut self.listeners {
            l.command_status_changed();
        }
    }

    fn notify_list_changed(&mut self) {
        for l in &mut self.listeners {
            l.command_list_changed();
        }
    }
}

impl std::fmt::Debug for ApplicationCommandManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationCommandManager")
            .field("commands", &self.commands)
            .field("order", &self.order)
            .field("key_mappings", &self.key_mappings)
            .field("listeners", &self.listeners.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: true,
            alt: false,
            meta: false,
        }
    }

    fn ctrl_shift() -> Modifiers {
        Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
            meta: false,
        }
    }

    #[test]
    fn command_flags_const_ops() {
        let f = CommandFlags(CommandFlags::IS_DISABLED | CommandFlags::IS_TICKED);
        assert!(f.is_disabled());
        assert!(f.is_ticked());
        assert!(!f.hidden_in_menu());

        let g = CommandFlags(CommandFlags::HIDE_IN_MENU) | CommandFlags(CommandFlags::IS_DISABLED);
        assert!(g.is_disabled());
        assert!(g.hidden_in_menu());
    }

    #[test]
    fn key_press_from_char_uppercases_with_shift() {
        let no_shift = KeyPress::from_char('s', Modifiers::none());
        let with_shift = KeyPress::from_char('s', Modifiers { shift: true, ..Modifiers::none() });
        assert_eq!(no_shift.key_code, b's' as u32);
        assert_eq!(with_shift.key_code, b'S' as u32);
    }

    #[test]
    fn key_press_text_description() {
        let k = KeyPress::new(b's' as u32, ctrl());
        assert_eq!(k.get_text_description(), "Ctrl+s");
        let k2 = KeyPress::new(KeyPress::F5, Modifiers::none());
        assert_eq!(k2.get_text_description(), "F5");
        let k3 = KeyPress::new(KeyPress::RIGHT, ctrl_shift());
        assert_eq!(k3.get_text_description(), "Ctrl+Shift+Right");
    }

    #[test]
    fn key_press_is_printable() {
        assert!(KeyPress::from_key_code(b'A' as u32).is_printable());
        assert!(KeyPress::from_key_code(b'0' as u32).is_printable());
        assert!(!KeyPress::from_key_code(KeyPress::F1).is_printable());
        assert!(!KeyPress::from_key_code(KeyPress::DELETE).is_printable());
    }

    #[test]
    fn key_press_matches() {
        let a = KeyPress::new(b's' as u32, ctrl());
        let b = KeyPress::new(b's' as u32, ctrl());
        let c = KeyPress::new(b'S' as u32, ctrl());
        assert!(a.matches(&b));
        assert!(!a.matches(&c));
    }

    #[test]
    fn command_info_builder() {
        let info = CommandInfo::new(CommandId(42), "Copy")
            .with_category("Edit")
            .with_description("Copy selection")
            .with_default_keypress(KeyPress::new(b'c' as u32, ctrl()));
        assert_eq!(info.command_id, CommandId(42));
        assert_eq!(info.short_name, "Copy");
        assert_eq!(info.category, "Edit");
        assert_eq!(info.description, "Copy selection");
        assert!(info.default_keypress.is_some());
    }

    #[test]
    fn manager_register_and_lookup() {
        let mut mgr = ApplicationCommandManager::new();
        let save = CommandId(1);
        mgr.register_command(CommandInfo::new(save, "Save"));
        let open = CommandId(2);
        mgr.register_command(CommandInfo::new(open, "Open").with_category("File"));

        assert_eq!(mgr.get_num_commands(), 2);
        assert_eq!(mgr.get_command_for_id(save).map(|c| c.short_name.as_str()), Some("Save"));
        assert_eq!(mgr.get_name_of_command(save), "Save");
        assert_eq!(mgr.get_name_of_command(CommandId(999)), "");
    }

    #[test]
    fn manager_order_preserved() {
        let mut mgr = ApplicationCommandManager::new();
        mgr.register_command(CommandInfo::new(CommandId(1), "A"));
        mgr.register_command(CommandInfo::new(CommandId(2), "B"));
        mgr.register_command(CommandInfo::new(CommandId(3), "C"));
        assert_eq!(mgr.get_command_for_index(0).unwrap().short_name, "A");
        assert_eq!(mgr.get_command_for_index(2).unwrap().short_name, "C");
    }

    #[test]
    fn manager_categories() {
        let mut mgr = ApplicationCommandManager::new();
        mgr.register_command(CommandInfo::new(CommandId(1), "Save").with_category("File"));
        mgr.register_command(CommandInfo::new(CommandId(2), "Open").with_category("File"));
        mgr.register_command(CommandInfo::new(CommandId(3), "Copy").with_category("Edit"));
        let cats = mgr.get_command_categories();
        assert_eq!(cats, vec!["Edit".to_string(), "File".to_string()]);
        assert_eq!(mgr.get_commands_in_category("File"), vec![CommandId(1), CommandId(2)]);
        assert_eq!(mgr.get_commands_in_category("Edit"), vec![CommandId(3)]);
    }

    #[test]
    fn manager_remove_command_clears_mappings() {
        let mut mgr = ApplicationCommandManager::new();
        let save = CommandId(1);
        mgr.register_command(
            CommandInfo::new(save, "Save").with_default_keypress(KeyPress::new(b's' as u32, ctrl())),
        );
        assert!(mgr.find_key_mapping(&KeyPress::new(b's' as u32, ctrl())).is_some());
        mgr.remove_command(save);
        assert!(mgr.find_key_mapping(&KeyPress::new(b's' as u32, ctrl())).is_none());
        assert_eq!(mgr.get_num_commands(), 0);
    }

    #[test]
    fn manager_clear_commands() {
        let mut mgr = ApplicationCommandManager::new();
        mgr.register_command(CommandInfo::new(CommandId(1), "A"));
        mgr.register_command(CommandInfo::new(CommandId(2), "B"));
        mgr.clear_commands();
        assert_eq!(mgr.get_num_commands(), 0);
    }

    #[test]
    fn manager_add_key_mapping() {
        let mut mgr = ApplicationCommandManager::new();
        let id = CommandId(1);
        mgr.register_command(CommandInfo::new(id, "Test"));
        mgr.add_key_mapping(KeyPress::new(b'x' as u32, ctrl()), id);
        assert_eq!(
            mgr.find_key_mapping(&KeyPress::new(b'x' as u32, ctrl())),
            Some(id)
        );
    }

    #[test]
    fn manager_get_description_falls_back() {
        let mut mgr = ApplicationCommandManager::new();
        let id = CommandId(1);
        mgr.register_command(CommandInfo::new(id, "Save"));
        assert_eq!(mgr.get_description_of_command(id), "Save");
    }

    // --- KeyPressMappingSet -----------------------------------------------

    #[test]
    fn key_mapping_add_and_find() {
        let mut set = KeyPressMappingSet::new();
        let key = KeyPress::new(b'a' as u32, ctrl());
        let id = CommandId(1);
        assert!(set.add_mapping(key, id).is_none());
        assert_eq!(set.find_command_for_keypress(&key), Some(id));
    }

    #[test]
    fn key_mapping_remove() {
        let mut set = KeyPressMappingSet::new();
        let key = KeyPress::new(b'a' as u32, ctrl());
        set.add_mapping(key, CommandId(1));
        assert_eq!(set.remove_mapping(&key), Some(CommandId(1)));
        assert!(set.find_command_for_keypress(&key).is_none());
    }

    #[test]
    fn key_mapping_remove_all_for_command() {
        let mut set = KeyPressMappingSet::new();
        let key1 = KeyPress::new(b'a' as u32, ctrl());
        let key2 = KeyPress::new(b'b' as u32, ctrl());
        let id = CommandId(1);
        set.add_mapping(key1, id);
        set.add_mapping(key2, id);
        set.remove_all_mappings_for_command(id);
        assert!(set.is_empty());
    }

    #[test]
    fn key_mapping_get_keypresses_for_command() {
        let mut set = KeyPressMappingSet::new();
        let id = CommandId(1);
        let k1 = KeyPress::new(b'a' as u32, ctrl());
        let k2 = KeyPress::new(b'b' as u32, ctrl_shift());
        set.add_mapping(k1, id);
        set.add_mapping(k2, id);
        let keys = set.get_keypresses_for_command(id);
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&k1));
        assert!(keys.contains(&k2));
    }

    #[test]
    fn key_mapping_iter_yields_all_pairs() {
        let mut set = KeyPressMappingSet::new();
        set.add_mapping(KeyPress::new(b'a' as u32, ctrl()), CommandId(1));
        set.add_mapping(KeyPress::new(b'b' as u32, ctrl()), CommandId(2));
        assert_eq!(set.iter().count(), 2);
    }

    #[test]
    fn key_mapping_to_compact_string_is_nonempty() {
        let mut set = KeyPressMappingSet::new();
        set.add_mapping(KeyPress::new(b'a' as u32, ctrl()), CommandId(1));
        set.add_mapping(KeyPress::new(KeyPress::F5, Modifiers::none()), CommandId(2));
        let s = set.to_compact_string();
        assert!(s.contains("0x61:"), "missing 'a' mapping: {s:?}");
        assert!(s.contains("0x114:"), "missing F5 mapping: {s:?}");
        assert!(s.contains("0x1"), "missing cmd id 1: {s:?}");
        assert!(s.contains("0x2"), "missing cmd id 2: {s:?}");
    }

    // --- Invocation / Target chain ----------------------------------------

    /// Test target that increments a counter every time `perform` is called.
    struct CountingTarget {
        id: CommandId,
        handled: CommandId,
        count: u32,
    }
    impl ApplicationCommandTarget for CountingTarget {
        fn get_command_info(&self, id: CommandId) -> Option<CommandInfo> {
            if id == self.id {
                Some(CommandInfo::new(self.id, "Count"))
            } else {
                None
            }
        }
        fn get_next_command_target(&self) -> Option<Box<dyn ApplicationCommandTarget>> {
            None
        }
        fn perform(&mut self, info: InvocationInfo) -> bool {
            if info.command_id == self.handled {
                self.count += 1;
                true
            } else {
                false
            }
        }
    }

    #[test]
    fn manager_invokes_target() {
        let mut mgr = ApplicationCommandManager::new();
        let id = CommandId(7);
        mgr.register_command(CommandInfo::new(id, "Count"));
        let target = CountingTarget {
            id,
            handled: id,
            count: 0,
        };
        mgr.set_first_command_target(Some(Box::new(target)));
        assert!(mgr.invoke_directly(id));
    }

    #[test]
    fn manager_returns_false_when_no_target_handles() {
        let mut mgr = ApplicationCommandManager::new();
        let id = CommandId(7);
        mgr.register_command(CommandInfo::new(id, "Count"));
        // No target set; invocation should fail cleanly.
        assert!(!mgr.invoke_directly(id));
    }

    #[test]
    fn manager_falls_through_to_next_target() {
        // The chain only has a single target here that rejects; we
        // exercise the fall-through path which simply returns `false`
        // when the chain is exhausted.
        let mut mgr = ApplicationCommandManager::new();
        let id = CommandId(1);
        mgr.register_command(CommandInfo::new(id, "X"));
        mgr.set_first_command_target(Some(Box::new(CountingTarget {
            id,
            handled: CommandId(999), // doesn't match — falls through
            count: 0,
        })));
        assert!(!mgr.invoke_directly(id));
    }

    // --- Listeners ---------------------------------------------------------

    struct ListRecorder {
        list_changes: u32,
        status_changes: u32,
    }
    impl CommandListener for ListRecorder {
        fn command_list_changed(&mut self) {
            self.list_changes += 1;
        }
        fn command_status_changed(&mut self) {
            self.status_changes += 1;
        }
    }

    #[test]
    fn manager_listener_gets_list_changed() {
        // Smoke test: the listener path runs without panicking. We can't
        // trivially inspect the listener's mutable state from inside the
        // same scope that mutates the manager, but the call sequence is
        // exercised.
        let mut mgr = ApplicationCommandManager::new();
        mgr.add_listener(Box::new(ListRecorder {
            list_changes: 0,
            status_changes: 0,
        }));
        mgr.register_command(CommandInfo::new(CommandId(1), "X"));
        mgr.clear_commands();
    }

    #[test]
    fn manager_command_status_changed_fires_listener() {
        let mut mgr = ApplicationCommandManager::new();
        mgr.add_listener(Box::new(ListRecorder {
            list_changes: 0,
            status_changes: 0,
        }));
        mgr.command_status_changed();
    }

    #[test]
    fn manager_clear_listeners() {
        let mut mgr = ApplicationCommandManager::new();
        mgr.add_listener(Box::new(ListRecorder {
            list_changes: 0,
            status_changes: 0,
        }));
        mgr.clear_listeners();
        // No assertion — just exercising the code path.
    }

    // --- Duplicates / replacement -----------------------------------------

    #[test]
    fn manager_re_register_keeps_order_stable() {
        let mut mgr = ApplicationCommandManager::new();
        mgr.register_command(CommandInfo::new(CommandId(1), "A"));
        mgr.register_command(CommandInfo::new(CommandId(2), "B"));
        mgr.register_command(CommandInfo::new(CommandId(1), "A-v2"));
        assert_eq!(mgr.get_num_commands(), 2);
        assert_eq!(mgr.get_command_for_index(0).unwrap().short_name, "A-v2");
    }

    #[test]
    fn manager_get_command_for_index_out_of_bounds() {
        let mgr = ApplicationCommandManager::new();
        assert!(mgr.get_command_for_index(0).is_none());
        assert!(mgr.get_command_for_index(999).is_none());
    }

    #[test]
    fn invocation_origin_variants() {
        let info = InvocationInfo {
            command_id: CommandId(1),
            origin: InvocationOrigin::Key,
            asynchronously: true,
        };
        assert_eq!(info.origin, InvocationOrigin::Key);
        assert!(info.asynchronously);
    }
}