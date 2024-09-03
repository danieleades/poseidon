#![feature(impl_trait_in_assoc_type)]
use uuid::Uuid;
pub mod positions;
mod probability;

mod transmission_history;

mod coordinate;
pub use coordinate::Coordinate;

type NodeId = Uuid;

pub use positions::Positions;
