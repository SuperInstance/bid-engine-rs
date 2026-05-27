//! Bid model with amount, bidder, and validity.

use std::collections::HashMap;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// A single bid placed by a bidder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub bidder: String,
    pub amount: f64,
    #[serde(default)]
    pub auction_id: String,
    #[serde(default = "generate_id")]
    pub id: String,
    #[serde(default)]
    pub timestamp: f64,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub revoked: bool,
}

fn generate_id() -> String {
    format!("{:012x}", rand_value())
}

fn rand_value() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
}

impl Bid {
    pub fn new(bidder: &str, amount: f64) -> Self {
        let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
        Self {
            bidder: bidder.into(),
            amount,
            auction_id: String::new(),
            id: generate_id(),
            timestamp: ts,
            metadata: HashMap::new(),
            revoked: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && self.amount > 0.0
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for Bid {}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.amount.partial_cmp(&other.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bid_new() {
        let bid = Bid::new("alice", 100.0);
        assert_eq!(bid.bidder, "alice");
        assert!((bid.amount - 100.0).abs() < 1e-10);
        assert!(bid.is_valid());
    }

    #[test]
    fn test_bid_revoke() {
        let mut bid = Bid::new("alice", 50.0);
        bid.revoke();
        assert!(!bid.is_valid());
    }

    #[test]
    fn test_bid_zero_invalid() {
        let bid = Bid::new("alice", 0.0);
        assert!(!bid.is_valid());
    }

    #[test]
    fn test_bid_negative_invalid() {
        let bid = Bid::new("alice", -10.0);
        assert!(!bid.is_valid());
    }
}
