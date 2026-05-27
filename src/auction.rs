//! Auction mechanisms — English, Dutch, Sealed-bid, Vickrey.

use serde::{Deserialize, Serialize};

use crate::bid::Bid;

/// Auction format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuctionType {
    English,
    Dutch,
    Sealed,
    Vickrey,
}

/// Auction lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuctionState {
    Pending,
    Open,
    Closed,
    Settled,
    Cancelled,
}

/// Outcome of a settled auction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionResult {
    pub auction_id: String,
    pub auction_type: AuctionType,
    pub winner: Option<String>,
    pub winning_amount: f64,
    pub clearing_price: f64,
    pub bids_evaluated: usize,
    pub settled_at: Option<f64>,
}

/// An auction instance.
pub struct Auction {
    pub id: String,
    pub auction_type: AuctionType,
    pub state: AuctionState,
    pub reserve_price: f64,
    pub starting_price: f64,
    pub current_price: f64,
    pub decrement_step: f64,
    pub item_description: String,
    bids: Vec<Bid>,
    current_highest: Option<Bid>,
    result: Option<AuctionResult>,
}

impl Auction {
    pub fn new(auction_type: AuctionType) -> Self {
        let id = format!(
            "{:010x}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
                % 0xFFFFFFFFFF
        );
        Self {
            id,
            auction_type,
            state: AuctionState::Pending,
            reserve_price: 0.0,
            starting_price: 0.0,
            current_price: 0.0,
            decrement_step: 1.0,
            item_description: String::new(),
            bids: Vec::new(),
            current_highest: None,
            result: None,
        }
    }

    pub fn with_reserve(mut self, price: f64) -> Self {
        self.reserve_price = price;
        self
    }
    pub fn with_starting_price(mut self, price: f64) -> Self {
        self.starting_price = price;
        self.current_price = price;
        self
    }
    pub fn with_decrement(mut self, step: f64) -> Self {
        self.decrement_step = step;
        self
    }

    pub fn open(&mut self) -> Result<(), String> {
        if self.state != AuctionState::Pending {
            return Err(format!("Cannot open auction in state {:?}", self.state));
        }
        self.state = AuctionState::Open;
        Ok(())
    }

    pub fn place_bid(&mut self, mut bid: Bid) -> Result<(), String> {
        if self.state != AuctionState::Open {
            return Err(format!("Auction {} is not open", self.id));
        }
        if !bid.is_valid() {
            return Err("Invalid bid".into());
        }
        bid.auction_id = self.id.clone();

        match self.auction_type {
            AuctionType::English => {
                if let Some(ref highest) = self.current_highest {
                    if bid.amount <= highest.amount {
                        return Err(format!(
                            "English: bid {} must exceed current highest {}",
                            bid.amount, highest.amount
                        ));
                    }
                }
                self.current_price = bid.amount;
                self.current_highest = Some(bid.clone());
                self.bids.push(bid);
            }
            AuctionType::Dutch => {
                if !self.bids.is_empty() {
                    return Err("Dutch auction: already claimed".into());
                }
                self.current_highest = Some(bid.clone());
                self.bids.push(bid);
            }
            _ => {
                self.bids.push(bid);
            }
        }
        Ok(())
    }

    pub fn dutch_decrement(&mut self) -> Result<f64, String> {
        if self.auction_type != AuctionType::Dutch {
            return Err("Not a Dutch auction".into());
        }
        if !self.bids.is_empty() {
            return Err("Dutch auction already has a winning bid".into());
        }
        self.current_price = (self.current_price - self.decrement_step).max(0.0);
        Ok(self.current_price)
    }

    pub fn close(&mut self) -> Result<(), String> {
        if self.state != AuctionState::Open {
            return Err(format!("Cannot close in state {:?}", self.state));
        }
        self.state = AuctionState::Closed;
        Ok(())
    }

    pub fn settle(&mut self) -> Result<AuctionResult, String> {
        if self.state != AuctionState::Closed {
            return Err(format!("Cannot settle in state {:?}", self.state));
        }

        let valid: Vec<&Bid> = self.bids.iter().filter(|b| b.is_valid()).collect();

        let result = if valid.is_empty() {
            AuctionResult {
                auction_id: self.id.clone(),
                auction_type: self.auction_type,
                winner: None,
                winning_amount: 0.0,
                clearing_price: 0.0,
                bids_evaluated: 0,
                settled_at: None,
            }
        } else {
            match self.auction_type {
                AuctionType::English | AuctionType::Dutch => {
                    let best = valid
                        .iter()
                        .max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap())
                        .unwrap();
                    AuctionResult {
                        auction_id: self.id.clone(),
                        auction_type: self.auction_type,
                        winner: Some(best.bidder.clone()),
                        winning_amount: best.amount,
                        clearing_price: best.amount,
                        bids_evaluated: valid.len(),
                        settled_at: None,
                    }
                }
                AuctionType::Sealed => {
                    let best = valid
                        .iter()
                        .max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap())
                        .unwrap();
                    let (winner, price) = if best.amount >= self.reserve_price {
                        (Some(best.bidder.clone()), best.amount)
                    } else {
                        (None, 0.0)
                    };
                    AuctionResult {
                        auction_id: self.id.clone(),
                        auction_type: self.auction_type,
                        winner,
                        winning_amount: price,
                        clearing_price: price,
                        bids_evaluated: valid.len(),
                        settled_at: None,
                    }
                }
                AuctionType::Vickrey => {
                    let mut sorted: Vec<&&Bid> = valid.iter().collect();
                    sorted.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
                    let best = sorted[0];
                    let second_price = if sorted.len() > 1 {
                        sorted[1].amount
                    } else {
                        self.reserve_price
                    };
                    let payment = second_price.max(self.reserve_price);
                    AuctionResult {
                        auction_id: self.id.clone(),
                        auction_type: self.auction_type,
                        winner: Some(best.bidder.clone()),
                        winning_amount: payment,
                        clearing_price: payment,
                        bids_evaluated: valid.len(),
                        settled_at: None,
                    }
                }
            }
        };

        self.state = AuctionState::Settled;
        self.result = Some(result.clone());
        Ok(result)
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        if self.state == AuctionState::Settled {
            return Err("Cannot cancel a settled auction".into());
        }
        self.state = AuctionState::Cancelled;
        Ok(())
    }

    pub fn bids(&self) -> &[Bid] {
        &self.bids
    }
    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }
    pub fn get_result(&self) -> Option<&AuctionResult> {
        self.result.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sealed_auction() {
        let mut a = Auction::new(AuctionType::Sealed);
        a.open().unwrap();
        a.place_bid(Bid::new("alice", 100.0)).unwrap();
        a.place_bid(Bid::new("bob", 150.0)).unwrap();
        a.close().unwrap();
        let result = a.settle().unwrap();
        assert_eq!(result.winner.as_deref(), Some("bob"));
        assert!((result.winning_amount - 150.0).abs() < 1e-10);
    }

    #[test]
    fn test_vickrey_auction() {
        let mut a = Auction::new(AuctionType::Vickrey).with_reserve(50.0);
        a.open().unwrap();
        a.place_bid(Bid::new("alice", 100.0)).unwrap();
        a.place_bid(Bid::new("bob", 150.0)).unwrap();
        a.close().unwrap();
        let result = a.settle().unwrap();
        assert_eq!(result.winner.as_deref(), Some("bob"));
        // Winner pays second price
        assert!((result.winning_amount - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_english_auction() {
        let mut a = Auction::new(AuctionType::English).with_starting_price(50.0);
        a.open().unwrap();
        a.place_bid(Bid::new("alice", 75.0)).unwrap();
        a.place_bid(Bid::new("bob", 100.0)).unwrap();
        a.close().unwrap();
        let result = a.settle().unwrap();
        assert_eq!(result.winner.as_deref(), Some("bob"));
    }

    #[test]
    fn test_english_must_exceed() {
        let mut a = Auction::new(AuctionType::English);
        a.open().unwrap();
        a.place_bid(Bid::new("alice", 100.0)).unwrap();
        assert!(a.place_bid(Bid::new("bob", 50.0)).is_err());
    }

    #[test]
    fn test_dutch_auction() {
        let mut a = Auction::new(AuctionType::Dutch)
            .with_starting_price(100.0)
            .with_decrement(10.0);
        a.open().unwrap();
        a.dutch_decrement().unwrap();
        a.place_bid(Bid::new("alice", 90.0)).unwrap();
        a.close().unwrap();
        let result = a.settle().unwrap();
        assert_eq!(result.winner.as_deref(), Some("alice"));
    }

    #[test]
    fn test_no_bids() {
        let mut a = Auction::new(AuctionType::Sealed);
        a.open().unwrap();
        a.close().unwrap();
        let result = a.settle().unwrap();
        assert!(result.winner.is_none());
    }

    #[test]
    fn test_reserve_not_met() {
        let mut a = Auction::new(AuctionType::Sealed).with_reserve(200.0);
        a.open().unwrap();
        a.place_bid(Bid::new("alice", 100.0)).unwrap();
        a.close().unwrap();
        let result = a.settle().unwrap();
        assert!(result.winner.is_none());
    }

    #[test]
    fn test_cancel() {
        let mut a = Auction::new(AuctionType::Sealed);
        a.cancel().unwrap();
        assert_eq!(a.state, AuctionState::Cancelled);
    }
}
