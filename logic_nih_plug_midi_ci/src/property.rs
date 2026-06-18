//! Property-exchange accounting.
//!
//! This module tracks in-flight property exchange requests and subscriptions
//! that *we* initiated. Incoming requests use the same `RequestId` field but
//! are tracked separately by the [`crate::device::Device`] dispatcher.

use std::collections::HashMap;

use crate::types::{Muid, RequestId, RequestKey, SubscriptionKey};

/// A pending property-exchange request that this device initiated.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PendingRequest {
    /// The request id we generated.
    pub request_id: RequestId,
    /// The peer's MUID.
    pub muid: Muid,
}

/// A pending subscription that this device initiated.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PendingSubscription {
    /// The subscribe id returned by the responder, if known.
    pub subscribe_id: Option<u32>,
    /// The peer's MUID.
    pub muid: Muid,
    /// The resource name (e.g. `"ResourceList"`).
    pub resource: Option<String>,
}

/// In-memory ledger of outstanding property exchange transactions.
#[derive(Default, Debug, Clone)]
pub struct PropertyLedger {
    requests: HashMap<RequestKey, PendingRequest>,
    subscriptions: HashMap<SubscriptionKey, PendingSubscription>,
    next_request_key: u32,
    next_subscription_key: u32,
}

impl PropertyLedger {
    /// Create a new, empty ledger.
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            subscriptions: HashMap::new(),
            next_request_key: 1,
            next_subscription_key: 1,
        }
    }

    /// Allocate a fresh request key and record `request`.
    pub fn begin_request(&mut self, request: PendingRequest) -> RequestKey {
        let key = RequestKey(self.next_request_key);
        self.next_request_key = self.next_request_key.wrapping_add(1).max(1);
        self.requests.insert(key, request);
        key
    }

    /// Look up a pending request by key.
    pub fn request(&self, key: RequestKey) -> Option<&PendingRequest> {
        self.requests.get(&key)
    }

    /// Finish (forget) a pending request.
    pub fn end_request(&mut self, key: RequestKey) -> Option<PendingRequest> {
        self.requests.remove(&key)
    }

    /// Allocate a fresh subscription key and record `subscription`.
    pub fn begin_subscription(&mut self, subscription: PendingSubscription) -> SubscriptionKey {
        let key = SubscriptionKey(self.next_subscription_key);
        self.next_subscription_key = self.next_subscription_key.wrapping_add(1).max(1);
        self.subscriptions.insert(key, subscription);
        key
    }

    /// Look up a pending subscription.
    pub fn subscription(&self, key: SubscriptionKey) -> Option<&PendingSubscription> {
        self.subscriptions.get(&key)
    }

    /// Finish a subscription.
    pub fn end_subscription(&mut self, key: SubscriptionKey) -> Option<PendingSubscription> {
        self.subscriptions.remove(&key)
    }

    /// Update the subscribe id of an existing subscription.
    pub fn set_subscription_id(&mut self, key: SubscriptionKey, id: u32) {
        if let Some(entry) = self.subscriptions.get_mut(&key) {
            entry.subscribe_id = Some(id);
        }
    }

    /// All pending subscription keys.
    pub fn ongoing_subscriptions(&self) -> Vec<SubscriptionKey> {
        let mut keys: Vec<_> = self.subscriptions.keys().copied().collect();
        keys.sort_by_key(|k| k.0);
        keys
    }

    /// All pending request keys.
    pub fn ongoing_requests(&self) -> Vec<RequestKey> {
        let mut keys: Vec<_> = self.requests.keys().copied().collect();
        keys.sort_by_key(|k| k.0);
        keys
    }

    /// Whether the ledger has no outstanding transactions.
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty() && self.subscriptions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_lifecycle() {
        let mut ledger = PropertyLedger::new();
        let key = ledger.begin_request(PendingRequest {
            request_id: RequestId::new(1).unwrap(),
            muid: Muid::from_bits_truncate(0x0102_0304),
        });
        assert!(ledger.request(key).is_some());
        assert_eq!(ledger.ongoing_requests(), vec![key]);
        assert!(ledger.end_request(key).is_some());
        assert!(ledger.is_empty());
    }

    #[test]
    fn subscription_lifecycle() {
        let mut ledger = PropertyLedger::new();
        let key = ledger.begin_subscription(PendingSubscription {
            subscribe_id: None,
            muid: Muid::from_bits_truncate(0x0102_0304),
            resource: Some("ResourceList".to_string()),
        });
        ledger.set_subscription_id(key, 42);
        assert_eq!(ledger.subscription(key).and_then(|s| s.subscribe_id), Some(42));
        assert!(ledger.end_subscription(key).is_some());
    }

    #[test]
    fn keys_are_unique() {
        let mut ledger = PropertyLedger::new();
        let k1 = ledger.begin_request(PendingRequest {
            request_id: RequestId::new(1).unwrap(),
            muid: Muid::from_bits_truncate(1),
        });
        let k2 = ledger.begin_request(PendingRequest {
            request_id: RequestId::new(2).unwrap(),
            muid: Muid::from_bits_truncate(2),
        });
        assert_ne!(k1, k2);
    }
}