//! Auction history — tracking past auctions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::auction::{AuctionResult, AuctionType};

/// Historical record of a completed auction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionRecord {
    pub auction_id: String,
    pub auction_type: AuctionType,
    pub winner: Option<String>,
    pub winning_amount: f64,
    pub clearing_price: f64,
    pub bids_evaluated: usize,
    pub settled_at: Option<f64>,
}

/// Append-only log of auction outcomes.
#[derive(Debug, Clone, Default)]
pub struct AuctionHistory {
    records: Vec<AuctionRecord>,
}

impl AuctionHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, result: &AuctionResult) -> AuctionRecord {
        let rec = AuctionRecord {
            auction_id: result.auction_id.clone(),
            auction_type: result.auction_type,
            winner: result.winner.clone(),
            winning_amount: result.winning_amount,
            clearing_price: result.clearing_price,
            bids_evaluated: result.bids_evaluated,
            settled_at: result.settled_at,
        };
        self.records.push(rec.clone());
        rec
    }

    pub fn records(&self) -> &[AuctionRecord] {
        &self.records
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn by_winner(&self, bidder: &str) -> Vec<&AuctionRecord> {
        self.records
            .iter()
            .filter(|r| r.winner.as_deref() == Some(bidder))
            .collect()
    }

    pub fn by_type(&self, auction_type: AuctionType) -> Vec<&AuctionRecord> {
        self.records
            .iter()
            .filter(|r| r.auction_type == auction_type)
            .collect()
    }

    pub fn total_revenue(&self) -> f64 {
        self.records.iter().map(|r| r.winning_amount).sum()
    }

    pub fn average_clearing_price(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.records.iter().map(|r| r.clearing_price).sum::<f64>() / self.records.len() as f64
    }

    /// Top bidders by wins: (bidder, wins, total_spent).
    pub fn leaderboard(&self, limit: usize) -> Vec<(String, usize, f64)> {
        let mut stats: HashMap<String, (usize, f64)> = HashMap::new();
        for r in &self.records {
            if let Some(ref w) = r.winner {
                let entry = stats.entry(w.clone()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += r.winning_amount;
            }
        }
        let mut board: Vec<_> = stats
            .into_iter()
            .map(|(k, (wins, spent))| (k, wins, spent))
            .collect();
        board.sort_by(|a, b| b.1.cmp(&a.1));
        board.truncate(limit);
        board
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(winner: &str, amount: f64) -> AuctionResult {
        AuctionResult {
            auction_id: "test".into(),
            auction_type: AuctionType::Sealed,
            winner: Some(winner.into()),
            winning_amount: amount,
            clearing_price: amount,
            bids_evaluated: 2,
            settled_at: None,
        }
    }

    #[test]
    fn test_record_and_query() {
        let mut h = AuctionHistory::new();
        h.record(&make_result("alice", 100.0));
        h.record(&make_result("bob", 200.0));
        assert_eq!(h.len(), 2);
        assert!((h.total_revenue() - 300.0).abs() < 1e-10);
    }

    #[test]
    fn test_by_winner() {
        let mut h = AuctionHistory::new();
        h.record(&make_result("alice", 100.0));
        h.record(&make_result("alice", 150.0));
        h.record(&make_result("bob", 200.0));
        assert_eq!(h.by_winner("alice").len(), 2);
    }

    #[test]
    fn test_leaderboard() {
        let mut h = AuctionHistory::new();
        h.record(&make_result("alice", 100.0));
        h.record(&make_result("alice", 150.0));
        h.record(&make_result("bob", 200.0));
        let lb = h.leaderboard(10);
        assert_eq!(lb[0].0, "alice");
        assert_eq!(lb[0].1, 2);
    }

    #[test]
    fn test_average_clearing_price() {
        let mut h = AuctionHistory::new();
        h.record(&make_result("a", 100.0));
        h.record(&make_result("b", 200.0));
        assert!((h.average_clearing_price() - 150.0).abs() < 1e-10);
    }

    #[test]
    fn test_clear() {
        let mut h = AuctionHistory::new();
        h.record(&make_result("a", 100.0));
        h.clear();
        assert!(h.is_empty());
    }
}
