#![feature(impl_trait_in_assoc_type)]
use uuid::Uuid;
mod positions;
mod probability;

mod transmission_history;

mod coordinate;
pub use coordinate::Coordinate;

mod novelty;

type NodeId = Uuid;

pub use positions::Positions;
