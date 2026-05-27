//! Resource allocation based on auction results.

use serde::{Deserialize, Serialize};

use crate::auction::AuctionResult;
use crate::bid::Bid;

/// A single allocation to a bidder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub bidder: String,
    pub resource_id: String,
    pub amount: f64,
}

/// Complete allocation outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResult {
    pub allocations: Vec<Allocation>,
    pub unallocated_bidders: Vec<String>,
    pub total_allocated: f64,
}

impl AllocationResult {
    pub fn is_empty(&self) -> bool {
        self.allocations.is_empty()
    }
}

/// Distribute resources to bidders.
pub struct ResourceAllocator {
    pub total_resources: f64,
    pub resource_id: String,
}

impl ResourceAllocator {
    pub fn new(total_resources: f64, resource_id: &str) -> Self {
        Self {
            total_resources,
            resource_id: resource_id.into(),
        }
    }

    /// Give entire resource pool to auction winner.
    pub fn allocate_single_winner(&self, result: &AuctionResult) -> AllocationResult {
        match result.winner {
            Some(ref winner) => AllocationResult {
                allocations: vec![Allocation {
                    bidder: winner.clone(),
                    resource_id: self.resource_id.clone(),
                    amount: self.total_resources,
                }],
                unallocated_bidders: Vec::new(),
                total_allocated: self.total_resources,
            },
            None => AllocationResult {
                allocations: Vec::new(),
                unallocated_bidders: Vec::new(),
                total_allocated: 0.0,
            },
        }
    }

    /// Split resources proportionally by bid amounts.
    pub fn allocate_proportional(&self, bids: &[Bid]) -> AllocationResult {
        let valid: Vec<&Bid> = bids.iter().filter(|b| b.is_valid()).collect();
        if valid.is_empty() {
            return AllocationResult {
                allocations: Vec::new(),
                unallocated_bidders: Vec::new(),
                total_allocated: 0.0,
            };
        }
        let total_bid: f64 = valid.iter().map(|b| b.amount).sum();
        if total_bid <= 0.0 {
            return AllocationResult {
                allocations: Vec::new(),
                unallocated_bidders: Vec::new(),
                total_allocated: 0.0,
            };
        }
        let mut allocations = Vec::new();
        let mut total_allocated = 0.0;
        for b in &valid {
            let share = (b.amount / total_bid) * self.total_resources;
            total_allocated += share;
            allocations.push(Allocation {
                bidder: b.bidder.clone(),
                resource_id: self.resource_id.clone(),
                amount: share,
            });
        }
        AllocationResult {
            allocations,
            unallocated_bidders: Vec::new(),
            total_allocated,
        }
    }

    /// Give each winner a fixed-size bundle.
    pub fn allocate_fixed_bundle(&self, bids: &[Bid], bundle_size: f64) -> AllocationResult {
        let valid: Vec<&Bid> = bids.iter().filter(|b| b.is_valid()).collect();
        let mut remaining = self.total_resources;
        let mut allocations = Vec::new();
        let mut unallocated = Vec::new();
        // Sort by amount descending — highest bidders get bundles first
        let mut sorted = valid.clone();
        sorted.sort_by(|a, b| {
            b.amount
                .partial_cmp(&a.amount)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for b in &sorted {
            if remaining >= bundle_size {
                allocations.push(Allocation {
                    bidder: b.bidder.clone(),
                    resource_id: self.resource_id.clone(),
                    amount: bundle_size,
                });
                remaining -= bundle_size;
            } else {
                unallocated.push(b.bidder.clone());
            }
        }
        AllocationResult {
            total_allocated: self.total_resources - remaining,
            allocations,
            unallocated_bidders: unallocated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_winner() {
        let result = AuctionResult {
            auction_id: "a1".into(),
            auction_type: crate::auction::AuctionType::Sealed,
            winner: Some("alice".into()),
            winning_amount: 100.0,
            clearing_price: 100.0,
            bids_evaluated: 2,
            settled_at: None,
        };
        let alloc = ResourceAllocator::new(1000.0, "gpu");
        let out = alloc.allocate_single_winner(&result);
        assert_eq!(out.allocations.len(), 1);
        assert!((out.allocations[0].amount - 1000.0).abs() < 1e-10);
    }

    #[test]
    fn test_proportional() {
        let bids = vec![Bid::new("alice", 100.0), Bid::new("bob", 300.0)];
        let alloc = ResourceAllocator::new(1000.0, "gpu");
        let out = alloc.allocate_proportional(&bids);
        assert_eq!(out.allocations.len(), 2);
        assert!((out.allocations[0].amount - 250.0).abs() < 1e-10);
        assert!((out.allocations[1].amount - 750.0).abs() < 1e-10);
    }

    #[test]
    fn test_fixed_bundle() {
        let bids = vec![
            Bid::new("alice", 200.0),
            Bid::new("bob", 100.0),
            Bid::new("carol", 50.0),
        ];
        let alloc = ResourceAllocator::new(200.0, "gpu");
        let out = alloc.allocate_fixed_bundle(&bids, 100.0);
        assert_eq!(out.allocations.len(), 2);
        assert_eq!(out.unallocated_bidders.len(), 1);
    }
}
