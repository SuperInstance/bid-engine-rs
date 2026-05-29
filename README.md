# bid-engine-rs

<<<<<<< HEAD
Rust port of [bid-engine](https://github.com/SuperInstance/bid-engine) — auction engine with English, Dutch, Sealed-bid, and Vickrey auctions.

## Features

- **English auction** — ascending open-cry
- **Dutch auction** — descending price, first bidder wins
- **Sealed-bid auction** — first-price sealed-bid
- **Vickrey auction** — second-price sealed-bid (incentive-compatible)
- **BidEvaluator** — scoring, ranking, filtering bids
- **ResourceAllocator** — single-winner, proportional, fixed-bundle allocation
- **AuctionHistory** — append-only log with leaderboard and revenue tracking
=======
Rust port of [bid-engine](https://github.com/SuperInstance/bid-engine) — auction resolution engine.

## Features

- **First-price auction**: winner pays their bid
- **Second-price / Vickrey auction**: winner pays second-highest bid
- **Multi-unit auction**: uniform price for N items
- **Reserve price filtering**
- **Bid statistics**: mean, median, std dev, spread
- **Bid shading detection**: identify suspiciously low bids
>>>>>>> 318a237 (Initial Rust port: auction bid engine)

## Usage

```rust
<<<<<<< HEAD
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
=======
use bid_engine::{resolve_auction, AuctionType, Bid};

let bids = vec![
    Bid::new(1, 100.0),
    Bid::new(2, 150.0),
    Bid::new(3, 120.0),
];

let result = resolve_auction(&bids, AuctionType::SecondPrice);
assert_eq!(result.winner_id, Some(2)); // highest bidder wins
assert!((result.clearing_price - 120.0).abs() < 0.01); // pays second price
>>>>>>> 318a237 (Initial Rust port: auction bid engine)
```

## License

MIT

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance/OpenConstruct) ecosystem.
