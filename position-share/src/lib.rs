//! A library for sharing positional data between nodes in a network over
//! extremely limited bandwidth.
//!
//! `position-share` uses the concept of 'geometric' novelty to determine the
//! most important data to send over a constrained channel.
//!
//! It also has support for situations where nodes have incomplete knowledge of
//! which data points the other nodes have already received.

use uuid::Uuid;
mod positions;
mod probability;

mod transmission_history;

mod coordinate;
pub use coordinate::Coordinate;

pub type NodeId = Uuid;

pub use positions::{
    geometric_novelty::{rdp, rdp_area, GeometricNovelty},
    search_strategy::{Search, SearchStrategy},
    Positions,
};
