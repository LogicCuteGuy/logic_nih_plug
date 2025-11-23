//! ValueTree implementation.
//!
//! A hierarchical data structure with property management and change notifications.

use crate::error::DataError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Serializable representation of a ValueTree (without listeners).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct SerializableValueTree {
    type_name: String,
    properties: HashMap<String, Value>,
    children: Vec<SerializableValueTree>,
}

/// A value that can be stored in a ValueTree property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// Integer value.
    Int(i32),
    /// Floating point value.
    Float(f32),
    /// String value.
    String(String),
    /// Boolean value.
    Bool(bool),
}

/// Trait for listening to ValueTree changes.
pub trait ValueTreeListener: Send {
    /// Called when a property value changes.
    fn value_changed(&mut self, tree: &ValueTree, property: &str);
    
    /// Called when a child is added.
    fn child_added(&mut self, parent: &ValueTree, child: &ValueTree);
    
    /// Called when a child is removed.
    fn child_removed(&mut self, parent: &ValueTree, child: &ValueTree);
}

/// A hierarchical data structure for storing application state.
///
/// ValueTree provides:
/// - Hierarchical organization of data
/// - Type-safe property storage
/// - Change notifications via listeners
/// - XML serialization/deserialization
///
/// # Examples
///
/// ```
/// use nih_plug_data::{ValueTree, Value};
///
/// let mut tree = ValueTree::new("root");
/// tree.set_property("name", Value::String("test".to_string()));
/// 
/// let value = tree.get_property("name").unwrap();
/// ```
pub struct ValueTree {
    type_name: String,
    properties: HashMap<String, Value>,
    children: Vec<ValueTree>,
    listeners: Arc<Mutex<Vec<Box<dyn ValueTreeListener>>>>,
}

impl Clone for ValueTree {
    fn clone(&self) -> Self {
        Self {
            type_name: self.type_name.clone(),
            properties: self.properties.clone(),
            children: self.children.clone(),
            // Don't clone listeners - new instance has no listeners
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ValueTree {
    /// Creates a new ValueTree with the given type name.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::ValueTree;
    ///
    /// let tree = ValueTree::new("root");
    /// ```
    pub fn new(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
            properties: HashMap::new(),
            children: Vec::new(),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns the type name of this ValueTree.
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Sets a property value.
    ///
    /// If the property already exists, it will be updated and listeners will be notified.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{ValueTree, Value};
    ///
    /// let mut tree = ValueTree::new("root");
    /// tree.set_property("count", Value::Int(42));
    /// ```
    pub fn set_property(&mut self, name: &str, value: Value) {
        self.properties.insert(name.to_string(), value);
        self.notify_value_changed(name);
    }

    /// Gets a property value by name.
    ///
    /// Returns `None` if the property doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{ValueTree, Value};
    ///
    /// let mut tree = ValueTree::new("root");
    /// tree.set_property("count", Value::Int(42));
    /// 
    /// let value = tree.get_property("count").unwrap();
    /// ```
    pub fn get_property(&self, name: &str) -> Option<&Value> {
        self.properties.get(name)
    }

    /// Adds a child ValueTree.
    ///
    /// Listeners will be notified of the addition.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::ValueTree;
    ///
    /// let mut parent = ValueTree::new("parent");
    /// let child = ValueTree::new("child");
    /// parent.add_child(child);
    /// ```
    pub fn add_child(&mut self, child: ValueTree) {
        self.notify_child_added(&child);
        self.children.push(child);
    }

    /// Removes a child at the given index.
    ///
    /// Returns the removed child, or `None` if the index is out of bounds.
    /// Listeners will be notified of the removal.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::ValueTree;
    ///
    /// let mut parent = ValueTree::new("parent");
    /// let child = ValueTree::new("child");
    /// parent.add_child(child);
    /// 
    /// let removed = parent.remove_child(0);
    /// assert!(removed.is_some());
    /// ```
    pub fn remove_child(&mut self, index: usize) -> Option<ValueTree> {
        if index < self.children.len() {
            let child = self.children.remove(index);
            self.notify_child_removed(&child);
            Some(child)
        } else {
            None
        }
    }

    /// Returns the number of children.
    pub fn num_children(&self) -> usize {
        self.children.len()
    }

    /// Gets a child by index.
    pub fn get_child(&self, index: usize) -> Option<&ValueTree> {
        self.children.get(index)
    }

    /// Adds a listener for change notifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{ValueTree, ValueTreeListener};
    ///
    /// struct MyListener;
    /// impl ValueTreeListener for MyListener {
    ///     fn value_changed(&mut self, _tree: &ValueTree, _property: &str) {}
    ///     fn child_added(&mut self, _parent: &ValueTree, _child: &ValueTree) {}
    ///     fn child_removed(&mut self, _parent: &ValueTree, _child: &ValueTree) {}
    /// }
    ///
    /// let mut tree = ValueTree::new("root");
    /// tree.add_listener(Box::new(MyListener));
    /// ```
    pub fn add_listener(&mut self, listener: Box<dyn ValueTreeListener>) {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.push(listener);
        }
    }

    /// Serializes the ValueTree to XML format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{ValueTree, Value};
    ///
    /// let mut tree = ValueTree::new("root");
    /// tree.set_property("name", Value::String("test".to_string()));
    /// 
    /// let xml = tree.to_xml();
    /// ```
    #[cfg(feature = "valuetree")]
    pub fn to_xml(&self) -> String {
        use quick_xml::Writer;
        use std::io::Cursor;

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        self.write_xml(&mut writer);
        
        let result = writer.into_inner().into_inner();
        String::from_utf8(result).unwrap_or_default()
    }

    #[cfg(feature = "valuetree")]
    fn write_xml<W: std::io::Write>(&self, writer: &mut quick_xml::Writer<W>) {
        use quick_xml::events::{BytesEnd, BytesStart, Event};

        let mut elem = BytesStart::new(&self.type_name);
        
        // Add properties as attributes with type prefix
        for (key, value) in &self.properties {
            let value_str = match value {
                Value::Int(i) => format!("i:{}", i),
                Value::Float(f) => format!("f:{}", f),
                Value::String(s) => format!("s:{}", s),
                Value::Bool(b) => format!("b:{}", b),
            };
            elem.push_attribute((key.as_str(), value_str.as_str()));
        }

        writer.write_event(Event::Start(elem.borrow())).ok();

        // Write children
        for child in &self.children {
            child.write_xml(writer);
        }

        writer.write_event(Event::End(BytesEnd::new(&self.type_name))).ok();
    }

    /// Deserializes a ValueTree from XML format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::ValueTree;
    ///
    /// let xml = r#"<root name="test"></root>"#;
    /// let tree = ValueTree::from_xml(xml).unwrap();
    /// assert_eq!(tree.type_name(), "root");
    /// ```
    #[cfg(feature = "valuetree")]
    pub fn from_xml(xml: &str) -> Result<Self, DataError> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut stack: Vec<ValueTree> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut tree = ValueTree::new(&name);

                    // Parse attributes as properties
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value_str = String::from_utf8_lossy(&attr.value).to_string();

                            // Parse based on type prefix
                            let value = if let Some(stripped) = value_str.strip_prefix("i:") {
                                stripped.parse::<i32>()
                                    .map(Value::Int)
                                    .unwrap_or_else(|_| Value::String(value_str.clone()))
                            } else if let Some(stripped) = value_str.strip_prefix("f:") {
                                stripped.parse::<f32>()
                                    .map(Value::Float)
                                    .unwrap_or_else(|_| Value::String(value_str.clone()))
                            } else if let Some(stripped) = value_str.strip_prefix("b:") {
                                stripped.parse::<bool>()
                                    .map(Value::Bool)
                                    .unwrap_or_else(|_| Value::String(value_str.clone()))
                            } else if let Some(stripped) = value_str.strip_prefix("s:") {
                                Value::String(stripped.to_string())
                            } else {
                                // Fallback for backward compatibility: try to infer type
                                if let Ok(i) = value_str.parse::<i32>() {
                                    Value::Int(i)
                                } else if let Ok(f) = value_str.parse::<f32>() {
                                    Value::Float(f)
                                } else if let Ok(b) = value_str.parse::<bool>() {
                                    Value::Bool(b)
                                } else {
                                    Value::String(value_str)
                                }
                            };

                            tree.set_property(&key, value);
                        }
                    }

                    stack.push(tree);
                }
                Ok(Event::End(_)) => {
                    if stack.len() > 1 {
                        let child = stack.pop().unwrap();
                        if let Some(parent) = stack.last_mut() {
                            parent.add_child(child);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DataError::InvalidXml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        stack.pop().ok_or_else(|| DataError::InvalidXml("Empty XML".to_string()))
    }

    /// Serializes the ValueTree to binary format using bincode.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{ValueTree, Value};
    ///
    /// let mut tree = ValueTree::new("root");
    /// tree.set_property("name", Value::String("test".to_string()));
    /// 
    /// let binary = tree.to_binary().unwrap();
    /// ```
    pub fn to_binary(&self) -> Result<Vec<u8>, DataError> {
        let serializable = self.to_serializable();
        bincode::serialize(&serializable)
            .map_err(|e| DataError::InvalidXml(format!("Binary serialization failed: {}", e)))
    }

    /// Deserializes a ValueTree from binary format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_data::{ValueTree, Value};
    ///
    /// let mut tree = ValueTree::new("root");
    /// tree.set_property("name", Value::String("test".to_string()));
    /// 
    /// let binary = tree.to_binary().unwrap();
    /// let restored = ValueTree::from_binary(&binary).unwrap();
    /// assert_eq!(restored.type_name(), "root");
    /// ```
    pub fn from_binary(data: &[u8]) -> Result<Self, DataError> {
        let serializable: SerializableValueTree = bincode::deserialize(data)
            .map_err(|e| DataError::InvalidXml(format!("Binary deserialization failed: {}", e)))?;
        Ok(Self::from_serializable(serializable))
    }

    // Convert to serializable representation
    fn to_serializable(&self) -> SerializableValueTree {
        SerializableValueTree {
            type_name: self.type_name.clone(),
            properties: self.properties.clone(),
            children: self.children.iter().map(|c| c.to_serializable()).collect(),
        }
    }

    // Convert from serializable representation
    fn from_serializable(s: SerializableValueTree) -> Self {
        Self {
            type_name: s.type_name,
            properties: s.properties,
            children: s.children.into_iter().map(Self::from_serializable).collect(),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Notification helpers
    fn notify_value_changed(&self, property: &str) {
        if let Ok(mut listeners) = self.listeners.lock() {
            for listener in listeners.iter_mut() {
                listener.value_changed(self, property);
            }
        }
    }

    fn notify_child_added(&self, child: &ValueTree) {
        if let Ok(mut listeners) = self.listeners.lock() {
            for listener in listeners.iter_mut() {
                listener.child_added(self, child);
            }
        }
    }

    fn notify_child_removed(&self, child: &ValueTree) {
        if let Ok(mut listeners) = self.listeners.lock() {
            for listener in listeners.iter_mut() {
                listener.child_removed(self, child);
            }
        }
    }
}
