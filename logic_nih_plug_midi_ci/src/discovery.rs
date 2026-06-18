//! Discovery state and helpers.
//!
//! This module owns the per-device cache of last-known peer discovery info.
//! The actual message dispatch is done by [`crate::device::Device`]; this
//! module only stores the state.

use std::collections::HashMap;

use crate::types::{DeviceInfo, Muid};

/// Cached information about a single discovered peer.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PeerDiscovery {
    /// The peer's own MUID — the one it advertises in the discovery reply.
    pub muid: Muid,
    /// The peer's advertised device info.
    pub device_info: DeviceInfo,
    /// The peer's advertised maximum SysEx size.
    pub maximum_sysex_size: u32,
    /// The peer's advertised output path id (only meaningful for V2+).
    pub output_path_id: u8,
}

/// In-memory cache of every peer we have discovered.
#[derive(Default, Debug, Clone)]
pub struct DiscoveryState {
    peers: HashMap<Muid, PeerDiscovery>,
}

impl DiscoveryState {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record or update a peer.
    pub fn insert(&mut self, peer: PeerDiscovery) {
        self.peers.insert(peer.muid, peer);
    }

    /// Remove a peer.
    pub fn remove(&mut self, muid: Muid) -> Option<PeerDiscovery> {
        self.peers.remove(&muid)
    }

    /// Look up a peer.
    pub fn get(&self, muid: Muid) -> Option<&PeerDiscovery> {
        self.peers.get(&muid)
    }

    /// Iterate over all peers.
    pub fn iter(&self) -> impl Iterator<Item = (&Muid, &PeerDiscovery)> {
        self.peers.iter()
    }

    /// Number of known peers.
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// Whether we have any peers cached.
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// Return a list of all MUIDs we have heard from.
    pub fn discovered_muids(&self) -> Vec<Muid> {
        self.peers.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup() {
        let mut state = DiscoveryState::new();
        let peer = PeerDiscovery {
            muid: Muid::from_bits_truncate(0x0102_0304),
            device_info: DeviceInfo::example(),
            maximum_sysex_size: 1024,
            output_path_id: 0,
        };
        state.insert(peer);
        assert_eq!(state.get(peer.muid), Some(&peer));
        assert_eq!(state.discovered_muids(), vec![peer.muid]);
    }

    #[test]
    fn remove_clears_entry() {
        let mut state = DiscoveryState::new();
        let peer = PeerDiscovery {
            muid: Muid::from_bits_truncate(0x0102_0304),
            device_info: DeviceInfo::example(),
            maximum_sysex_size: 1024,
            output_path_id: 0,
        };
        state.insert(peer);
        assert_eq!(state.remove(peer.muid), Some(peer));
        assert!(state.is_empty());
    }
}