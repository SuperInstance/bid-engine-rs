//! Bid evaluation — scoring, ranking, filtering.

use serde::{Deserialize, Serialize};

use crate::bid::Bid;

/// Built-in scoring strategies.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreStrategy {
    AmountDesc,
    AmountAsc,
    Recency,
    Confidence,
}

/// A bid with its computed score and rank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedBid {
    pub bid: Bid,
    pub score: f64,
    pub rank: usize,
}

/// Score and rank bids.
pub struct BidEvaluator {
    pub strategy: ScoreStrategy,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
}

impl BidEvaluator {
    pub fn new(strategy: ScoreStrategy) -> Self {
        Self {
            strategy,
            min_amount: None,
            max_amount: None,
        }
    }

    pub fn with_min_amount(mut self, min: f64) -> Self {
        self.min_amount = Some(min);
        self
    }
    pub fn with_max_amount(mut self, max: f64) -> Self {
        self.max_amount = Some(max);
        self
    }

    pub fn filter_bids<'a>(&self, bids: &'a [Bid]) -> Vec<&'a Bid> {
        bids.iter()
            .filter(|b| {
                if !b.is_valid() {
                    return false;
                }
                if let Some(min) = self.min_amount {
                    if b.amount < min {
                        return false;
                    }
                }
                if let Some(max) = self.max_amount {
                    if b.amount > max {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    fn score(&self, bid: &Bid) -> f64 {
        match self.strategy {
            ScoreStrategy::AmountDesc => bid.amount,
            ScoreStrategy::AmountAsc => -bid.amount,
            ScoreStrategy::Recency => -bid.timestamp,
            ScoreStrategy::Confidence => bid
                .metadata
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        }
    }

    pub fn rank_bids(&self, bids: &[Bid]) -> Vec<RankedBid> {
        let filtered = self.filter_bids(bids);
        let mut scored: Vec<_> = filtered.into_iter().map(|b| (b, self.score(b))).collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .enumerate()
            .map(|(i, (bid, score))| RankedBid {
                bid: bid.clone(),
                score,
                rank: i + 1,
            })
            .collect()
    }

    pub fn top_n(&self, bids: &[Bid], n: usize) -> Vec<RankedBid> {
        let mut ranked = self.rank_bids(bids);
        ranked.truncate(n);
        ranked
    }

    pub fn winner(&self, bids: &[Bid]) -> Option<RankedBid> {
        self.rank_bids(bids).into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bids() -> Vec<Bid> {
        vec![
            Bid::new("alice", 100.0),
            Bid::new("bob", 200.0),
            Bid::new("carol", 150.0),
        ]
    }

    #[test]
    fn test_rank_amount_desc() {
        let eval = BidEvaluator::new(ScoreStrategy::AmountDesc);
        let ranked = eval.rank_bids(&make_bids());
        assert_eq!(ranked[0].bid.bidder, "bob");
        assert_eq!(ranked[0].rank, 1);
    }

    #[test]
    fn test_rank_amount_asc() {
        let eval = BidEvaluator::new(ScoreStrategy::AmountAsc);
        let ranked = eval.rank_bids(&make_bids());
        assert_eq!(ranked[0].bid.bidder, "alice");
    }

    #[test]
    fn test_filter_min_amount() {
        let eval = BidEvaluator::new(ScoreStrategy::AmountDesc).with_min_amount(120.0);
        let bids = make_bids();
        let filtered = eval.filter_bids(&bids);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_top_n() {
        let eval = BidEvaluator::new(ScoreStrategy::AmountDesc);
        let top = eval.top_n(&make_bids(), 2);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_winner() {
        let eval = BidEvaluator::new(ScoreStrategy::AmountDesc);
        let w = eval.winner(&make_bids()).unwrap();
        assert_eq!(w.bid.bidder, "bob");
    }
}
