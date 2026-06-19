//! Editor UI scaffolding.
//!
//! The actual `egui`-based editor implementation is feature-gated and
//! left as a documented pointer to `examples/Plugins/HostPluginDemo.h`
//! from JUCE. This module provides the **plumbing** (parameter
//! binding, state save/load) so the rest of the host can be tested
//! without instantiating an `egui` window.

/// A parameter binding — connects a slider in the editor to a host
/// parameter on the loaded plugin.
///
/// In a real impl, this would hold an
/// `Arc<AtomicFloat>` (the param's current value) + a callback for
/// user-driven changes. We model the same shape here without the
/// `egui` dependency, so the binding logic is testable.
#[derive(Debug, Clone)]
pub struct ParamBinding {
    /// Parameter ID (matches `#[id = "..."]` in the plugin).
    pub param_id: String,
    /// Display label shown next to the slider.
    pub label: String,
    /// Current value (linear).
    pub value: f32,
    /// Minimum value.
    pub min: f32,
    /// Maximum value.
    pub max: f32,
    /// Default value (used by "Reset to default").
    pub default: f32,
}

impl ParamBinding {
    /// Create a new binding. The `value` is clamped to `[min, max]`.
    pub fn new(
        param_id: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        min: f32,
        max: f32,
        default: f32,
    ) -> Self {
        let mut s = Self {
            param_id: param_id.into(),
            label: label.into(),
            value,
            min,
            max,
            default,
        };
        s.clamp();
        s
    }

    /// Set the value (clamped to `[min, max]`).
    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    /// Normalized value in `[0.0, 1.0]`.
    pub fn normalized(&self) -> f32 {
        if self.max > self.min {
            (self.value - self.min) / (self.max - self.min)
        } else {
            0.0
        }
    }

    /// Set from a normalized value in `[0.0, 1.0]`.
    pub fn set_normalized(&mut self, n: f32) {
        let n = n.clamp(0.0, 1.0);
        self.set(self.min + n * (self.max - self.min));
    }

    /// Reset to default.
    pub fn reset_to_default(&mut self) {
        self.value = self.default;
    }

    fn clamp(&mut self) {
        self.value = self.value.clamp(self.min, self.max);
        self.default = self.default.clamp(self.min, self.max);
    }
}

/// State for the host editor — holds all parameter bindings + the
/// currently loaded plugin path. Saved/loaded to disk via the
/// `save_state` / `load_state` methods.
#[derive(Debug, Clone, Default)]
pub struct EditorState {
    /// All parameter bindings, keyed by `param_id`.
    pub params: Vec<ParamBinding>,
    /// Path of the currently loaded plugin (if any).
    pub loaded_plugin: Option<String>,
}

impl EditorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a parameter binding.
    pub fn add_param(&mut self, binding: ParamBinding) {
        self.params.push(binding);
    }

    /// Look up a binding by `param_id`.
    pub fn get_param(&self, param_id: &str) -> Option<&ParamBinding> {
        self.params.iter().find(|p| p.param_id == param_id)
    }

    /// Look up a binding by `param_id` (mutable).
    pub fn get_param_mut(&mut self, param_id: &str) -> Option<&mut ParamBinding> {
        self.params.iter_mut().find(|p| p.param_id == param_id)
    }

    /// Serialize the editor state to a JSON string for persistence.
    pub fn save_state(&self) -> String {
        let entries: Vec<String> = self
            .params
            .iter()
            .map(|p| {
                format!(
                    "{{\"id\":\"{}\",\"value\":{}}}",
                    p.param_id, p.value
                )
            })
            .collect();
        format!(
            "{{\"plugin\":\"{}\",\"params\":[{}]}}",
            self.loaded_plugin.as_deref().unwrap_or(""),
            entries.join(",")
        )
    }

    /// Parse a JSON state string produced by `save_state` and apply
    /// the values to the current bindings. Returns `true` on success.
    pub fn load_state(&mut self, json: &str) -> bool {
        // Tiny hand-rolled parser: we don't pull in `serde_json` just
        // for this demo. Look for `"id":"..."` and `"value":<float>`
        // pairs in the order they appear.
        let mut i = 0;
        let bytes = json.as_bytes();
        while i < bytes.len() {
            if let Some(start) = json[i..].find("\"id\":\"") {
                let id_start = i + start + 6;
                if let Some(end) = json[id_start..].find('"') {
                    let id = &json[id_start..id_start + end];
                    if let Some(value_start) = json[id_start + end..].find("\"value\":") {
                        let vs = id_start + end + value_start + 8;
                        let value_str: String = json[vs..]
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                            .collect();
                        if let Ok(v) = value_str.parse::<f32>() {
                            if let Some(p) = self.get_param_mut(id) {
                                p.set(v);
                            }
                        }
                    }
                    i = id_start + end + 1;
                    continue;
                }
            }
            break;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn param_binding_clamps_value() {
        let p = ParamBinding::new("gain", "Gain", 5.0, 0.0, 1.0, 0.5);
        assert_eq!(p.value, 1.0);
    }

    #[test]
    fn param_binding_normalized_roundtrip() {
        let mut p = ParamBinding::new("gain", "Gain", 0.5, 0.0, 1.0, 0.5);
        p.set_normalized(0.25);
        assert!((p.value - 0.25).abs() < 1e-6);
        let n = p.normalized();
        assert!((n - 0.25).abs() < 1e-6);
    }

    #[test]
    fn editor_state_save_and_load_roundtrip() {
        let mut state = EditorState::new();
        state.add_param(ParamBinding::new("gain", "Gain", 0.75, 0.0, 1.0, 0.5));
        state.add_param(ParamBinding::new("drive", "Drive", 0.3, 0.0, 1.0, 0.5));
        state.loaded_plugin = Some("/path/to/plugin.vst3".to_string());

        let json = state.save_state();
        assert!(json.contains("\"id\":\"gain\""));
        assert!(json.contains("\"value\":0.75"));

        let mut other = EditorState::new();
        other.add_param(ParamBinding::new("gain", "Gain", 0.0, 0.0, 1.0, 0.5));
        other.add_param(ParamBinding::new("drive", "Drive", 0.0, 0.0, 1.0, 0.5));
        assert!(other.load_state(&json));

        assert_eq!(other.get_param("gain").unwrap().value, 0.75);
        assert_eq!(other.get_param("drive").unwrap().value, 0.3);
    }
}
