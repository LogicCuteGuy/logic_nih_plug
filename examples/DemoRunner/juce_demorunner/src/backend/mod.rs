//! Backend dispatch — selects the active backend at runtime based on
//! the compile-time feature flags.

use crate::BackendKind;

/// The selected backend, ready to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveBackend {
    /// Which backend is active.
    pub kind: BackendKind,
}

impl ActiveBackend {
    /// Resolve the active backend for this build.
    pub fn resolve() -> Self {
        Self {
            kind: BackendKind::current(),
        }
    }
}

impl Default for ActiveBackend {
    fn default() -> Self {
        Self::resolve()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_backend_resolves_to_egui_for_default_build() {
        let backend = ActiveBackend::resolve();
        assert_eq!(backend.kind, BackendKind::Egui);
    }
}