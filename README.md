# bid-engine-rs

Auction resolution engine in Rust — English (ascending), Dutch (descending), sealed-bid (first-price), and Vickrey (second-price) auctions with bid evaluation, resource allocation, and full history tracking.

## What This Gives You

- **Four auction types**: English (ascending open-cry), Dutch (descending, first bidder wins), sealed-bid (first-price), and Vickrey (second-price, incentive-compatible)
- **Bid evaluation**: score, rank, and filter bids with configurable criteria
- **Resource allocation**: single-winner, proportional, and fixed-bundle allocation strategies
- **Auction history**: append-only log with leaderboard, revenue tracking, and settlement records
- **Reserve prices**: filter bids below minimum threshold
- **Full lifecycle**: Pending → Open → Closed → Settled with state machine enforcement

## Quick Start

```rust
use bid_engine::{Auction, AuctionType, Bid};

// Vickrey auction: winner pays second-highest bid
let mut auction = Auction::new(AuctionType::Vickrey).with_reserve(50.0);
auction.open().unwrap();

auction.place_bid(Bid::new("alice", 100.0)).unwrap();
auction.place_bid(Bid::new("bob", 150.0)).unwrap();
auction.place_bid(Bid::new("carol", 120.0)).unwrap();

auction.close().unwrap();
let result = auction.settle().unwrap();

assert_eq!(result.winner.as_deref(), Some("bob"));   // highest bidder wins
assert_eq!(result.winning_amount, 100.0);             // pays second price (alice's bid)
```

```rust
// Dutch auction: price descends, first bidder wins
let mut dutch = Auction::new(AuctionType::Dutch)
    .with_starting_price(200.0)
    .with_decrement(10.0);
dutch.open().unwrap();

// Price drops each round until someone bids
dutch.place_bid(Bid::new("buyer", 160.0)).unwrap();  // wins at current price
```

## API Reference

### Auction

```rust
impl Auction {
    pub fn new(auction_type: AuctionType) -> Self;
    pub fn with_reserve(self, price: f64) -> Self;
    pub fn with_starting_price(self, price: f64) -> Self;
    pub fn with_decrement(self, step: f64) -> Self;
    pub fn open(&mut self) -> Result<(), String>;
    pub fn place_bid(&mut self, bid: Bid) -> Result<(), String>;
    pub fn close(&mut self) -> Result<(), String>;
    pub fn settle(&mut self) -> Result<AuctionResult, String>;
    pub fn bid_count(&self) -> usize;
    pub fn current_price(&self) -> f64;
}
```

### AuctionType / AuctionState / AuctionResult

```rust
pub enum AuctionType { English, Dutch, Sealed, Vickrey }
pub enum AuctionState { Pending, Open, Closed, Settled, Cancelled }

pub struct AuctionResult {
    pub auction_id: String,
    pub auction_type: AuctionType,
    pub winner: Option<String>,
    pub winning_amount: f64,
    pub clearing_price: f64,
    pub bids_evaluated: usize,
}
```

### Bid

```rust
pub struct Bid {
    pub bidder: String,
    pub amount: f64,
    pub timestamp: f64,
}

impl Bid {
    pub fn new(bidder: &str, amount: f64) -> Self;
}
```

### Evaluation & Allocation

```rust
// BidEvaluator: score, rank, filter bids
// ResourceAllocator: single-winner, proportional, fixed-bundle
// AuctionHistory: append-only log with leaderboard and revenue
```

## How It Fits

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance/OpenConstruct) ecosystem. Used by:

- **agent-dna-rs** — genetic strategies for auction bidding
- **ab-testing-rs** — statistical evaluation of auction outcomes
- **caching-service-rs** — cache auction results for quick lookup

## Testing

**25 tests** covering all four auction types, reserve price filtering, bid evaluation, resource allocation strategies, history tracking, and state machine transitions (opening/closing/settling in wrong order).

## Installation

```toml
# Cargo.toml
[dependencies]
bid-engine = { git = "https://github.com/SuperInstance/bid-engine-rs" }
```

Requires Rust 2021 edition. Depends on `serde` and `serde_json` for serializable types.
