use std::collections::HashMap;

use uuid::Uuid;

use crate::{probability::Probability, NodeId};

/// Keeps track of the transmission history of a datum.
///
/// Records the probability that a datum has been successfully transmitted to a given recipient.
#[derive(Debug, Clone, Default)]
pub struct TransmissionHistory {
    /// Maps a recipient to a map of datums to their transmission probabilities.
    history: HashMap<NodeId, HashMap<Uuid, Probability>>,
}

impl TransmissionHistory {
    /// Returns the probability that a recipient has a datum.
    #[must_use]
    pub fn probability_recipient_has_datum(
        &self,
        recipient: &NodeId,
        datum_id: &Uuid,
    ) -> Probability {
        self.history
            .get(recipient)
            .and_then(|datums| datums.get(datum_id))
            .copied()
            .unwrap_or(Probability::ZERO)
    }
}
