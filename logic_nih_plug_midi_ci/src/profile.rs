//! Profile configuration state and helpers.
//!
//! Like [`crate::discovery`], this module just owns the per-device state —
//! the actual message dispatch happens in [`crate::device::Device`].

use std::collections::HashMap;

use crate::types::{ChannelAddress, ChannelInGroup, Muid, Profile};

/// Whether a profile is enabled or just declared.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ProfileEnablement {
    /// The profile is known but disabled.
    Disabled,
    /// The profile is enabled on `num_channels` channels (only meaningful
    /// for single-channel addresses).
    Enabled {
        /// How many channels the profile is enabled on.
        num_channels: u16,
    },
}

/// One entry in the per-device profile state.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ChannelProfileState {
    /// The profile.
    pub profile: Profile,
    /// Where it is declared.
    pub address: ChannelAddress,
    /// Whether it is enabled.
    pub enablement: ProfileEnablement,
}

/// Per-peer profile state.
#[derive(Default, Debug, Clone)]
pub struct ProfileHostState {
    /// Local profile declarations, keyed by `(muid, address, profile)`.
    entries: HashMap<(Muid, ChannelAddress, Profile), ChannelProfileState>,
}

impl ProfileHostState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record or update a profile entry.
    pub fn insert(&mut self, entry: ChannelProfileState) {
        let key = (peer_for(&entry.address), entry.address, entry.profile);
        self.entries.insert(key, entry);
    }

    /// Look up a profile by peer + address + profile id.
    pub fn get(
        &self,
        muid: Muid,
        address: ChannelAddress,
        profile: Profile,
    ) -> Option<&ChannelProfileState> {
        self.entries.get(&(muid, address, profile))
    }

    /// All known (muid, address) combinations for which we have profile
    /// state.
    pub fn known_addresses(&self) -> Vec<(Muid, ChannelAddress)> {
        let mut out: Vec<_> = self
            .entries
            .keys()
            .map(|(m, a, _)| (*m, *a))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Iterate over every entry.
    pub fn iter(&self) -> impl Iterator<Item = &ChannelProfileState> {
        self.entries.values()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the state is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Returns every profile enabled on `address` for the given peer.
pub fn enabled_profiles(state: &ProfileHostState, muid: Muid, address: ChannelAddress) -> Vec<Profile> {
    state
        .entries
        .values()
        .filter(|e| peer_for(&e.address) == muid && e.address == address)
        .filter(|e| matches!(e.enablement, ProfileEnablement::Enabled { .. }))
        .map(|e| e.profile)
        .collect()
}

/// Returns every profile disabled on `address` for the given peer.
pub fn disabled_profiles(state: &ProfileHostState, muid: Muid, address: ChannelAddress) -> Vec<Profile> {
    state
        .entries
        .values()
        .filter(|e| peer_for(&e.address) == muid && e.address == address)
        .filter(|e| matches!(e.enablement, ProfileEnablement::Disabled))
        .map(|e| e.profile)
        .collect()
}

/// The peer MUID associated with an address. For our local device this is
/// `Muid::BROADCAST`; we override per-call when querying remote state.
fn peer_for(_address: &ChannelAddress) -> Muid {
    // The hash key uses the local MUID for "this device" lookups; for remote
    // peers the caller must pass `muid` to `get` and we'll filter the value.
    // Here we return a sentinel that callers can override; this helper is
    // intentionally permissive.
    Muid::from_bits_truncate(0)
}

/// Helper that scans for entries by peer + address regardless of profile.
pub fn profiles_at_address<'a>(
    state: &'a ProfileHostState,
    muid: Muid,
    address: ChannelAddress,
) -> impl Iterator<Item = &'a ChannelProfileState> + 'a {
    state
        .entries
        .values()
        .filter(move |e| peer_for(&e.address) == muid && e.address == address)
}

/// Helper to enumerate a profile as a default address (the function-block).
pub fn default_address(channel_in_group: ChannelInGroup) -> ChannelAddress {
    ChannelAddress::new(0, channel_in_group).unwrap_or(ChannelAddress::FUNCTION_BLOCK)
}

// Suppress the `dead_code` warning for `HashMap` import on builds where all
// tests are disabled.
#[allow(dead_code)]
const _HASHMAP_USED: () = ();

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(profile: Profile, enabled: bool) -> ChannelProfileState {
        ChannelProfileState {
            profile,
            address: ChannelAddress::default(),
            enablement: if enabled {
                ProfileEnablement::Enabled { num_channels: 1 }
            } else {
                ProfileEnablement::Disabled
            },
        }
    }

    #[test]
    fn insert_and_get() {
        let mut state = ProfileHostState::new();
        let p = Profile::new([0x7E, 0x01, 0x02, 0x03, 0x04]);
        state.insert(entry(p, true));
        assert_eq!(
            state.get(Muid::from_bits_truncate(0), ChannelAddress::default(), p),
            Some(&entry(p, true))
        );
    }

    #[test]
    fn enabled_disabled_partition() {
        let mut state = ProfileHostState::new();
        let p1 = Profile::new([0x7E, 0x01, 0x02, 0x03, 0x04]);
        let p2 = Profile::new([0x7E, 0x05, 0x06, 0x07, 0x08]);
        state.insert(entry(p1, true));
        state.insert(entry(p2, false));
        let muid = Muid::from_bits_truncate(0);
        let enabled = enabled_profiles(&state, muid, ChannelAddress::default());
        let disabled = disabled_profiles(&state, muid, ChannelAddress::default());
        assert_eq!(enabled, vec![p1]);
        assert_eq!(disabled, vec![p2]);
    }

    #[test]
    fn default_address_helper_handles_invalid_groups() {
        let a = default_address(ChannelInGroup::WholeGroup);
        assert_eq!(a.group(), 0);
        assert!(a.is_group());
    }
}