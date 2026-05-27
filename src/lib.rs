//! bid-engine — Auction engine with English, Dutch, Sealed-bid, and Vickrey auctions.

pub mod bid;
pub mod auction;
pub mod evaluation;
pub mod allocation;
pub mod history;

pub use bid::Bid;
pub use auction::{Auction, AuctionType, AuctionState, AuctionResult};
