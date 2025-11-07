//! View key management for transaction privacy

use super::*;
use std::collections::{HashMap, HashSet};

/// View key manager
pub struct ViewManager {
    /// Authorized view keys per transaction
    authorized_views: HashMap<Hash, HashSet<StealthAddress>>,
}

impl ViewManager {
    /// Create a new view manager
    pub fn new() -> Self {
        Self {
            authorized_views: HashMap::new(),
        }
    }

    /// Authorize a view key for a transaction
    pub fn authorize(&mut self, view_key: StealthAddress, tx_hash: Hash) {
        self.authorized_views
            .entry(tx_hash)
            .or_insert_with(HashSet::new)
            .insert(view_key);
    }

    /// Check if a view key is authorized for a transaction
    pub fn is_authorized(&self, view_key: &StealthAddress, tx_hash: &Hash) -> bool {
        self.authorized_views
            .get(tx_hash)
            .map(|keys| keys.contains(view_key))
            .unwrap_or(false)
    }

    /// Revoke authorization for a transaction
    pub fn revoke(&mut self, view_key: &StealthAddress, tx_hash: &Hash) {
        if let Some(keys) = self.authorized_views.get_mut(tx_hash) {
            keys.remove(view_key);
            if keys.is_empty() {
                self.authorized_views.remove(tx_hash);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_authorization() {
        let mut manager = ViewManager::new();
        let view_key = StealthAddress::new();
        let tx_hash = [0; 32];

        // Initially not authorized
        assert!(!manager.is_authorized(&view_key, &tx_hash));

        // Authorize
        manager.authorize(view_key.clone(), tx_hash);
        assert!(manager.is_authorized(&view_key, &tx_hash));

        // Revoke
        manager.revoke(&view_key, &tx_hash);
        assert!(!manager.is_authorized(&view_key, &tx_hash));
    }
}