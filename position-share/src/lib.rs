#![feature(impl_trait_in_assoc_type)]
use uuid::Uuid;
mod positions;
mod probability;

mod transmission_history;
use transmission_history::TransmissionHistory;

mod coordinate;
pub use coordinate::Coordinate;

mod novelty;
use novelty::Novelty;

type NodeId = Uuid;

pub use positions::Positions;
