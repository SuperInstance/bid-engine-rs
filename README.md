# bid-engine-rs

Rust port of [bid-engine](https://github.com/SuperInstance/bid-engine) — auction engine with English, Dutch, Sealed-bid, and Vickrey auctions.

## Features

- **English auction** — ascending open-cry
- **Dutch auction** — descending price, first bidder wins
- **Sealed-bid auction** — first-price sealed-bid
- **Vickrey auction** — second-price sealed-bid (incentive-compatible)
- **BidEvaluator** — scoring, ranking, filtering bids
- **ResourceAllocator** — single-winner, proportional, fixed-bundle allocation
- **AuctionHistory** — append-only log with leaderboard and revenue tracking

## Usage

```rust
use bid_engine::{Auction, AuctionType, Bid};

let mut auction = Auction::new(AuctionType::Vickrey).with_reserve(50.0);
auction.open().unwrap();
auction.place_bid(Bid::new("alice", 100.0)).unwrap();
auction.place_bid(Bid::new("bob", 150.0)).unwrap();
auction.close().unwrap();
let result = auction.settle().unwrap();
// Bob wins, pays Alice's bid (second price) = 100
assert_eq!(result.winner.as_deref(), Some("bob"));
assert_eq!(result.winning_amount, 100.0);
```

## License

MIT
