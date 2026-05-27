//! bid-engine — Auction engine with English, Dutch, Sealed-bid, and Vickrey auctions.

pub mod allocation;
pub mod auction;
pub mod bid;
pub mod evaluation;
pub mod history;

pub use auction::{Auction, AuctionResult, AuctionState, AuctionType};
pub use bid::Bid;
