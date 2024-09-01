use std::cmp::Ordering;

use uuid::Uuid;

use crate::probability::Probability;

#[derive(Debug, PartialEq)]
pub struct Novelty {
    pub distance: f64,
    pub probability_already_transmitted: Probability,
    pub id: Uuid,
}

impl Novelty {
    pub fn score(&self) -> f64 {
        self.distance * self.probability_already_transmitted.complement()
    }
}

impl Ord for Novelty {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score()
            .partial_cmp(&other.score())
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                self.probability_already_transmitted
                    .cmp(&other.probability_already_transmitted)
            })
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for Novelty {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Novelty {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare() {
        let a = Novelty {
            distance: 2.0,
            probability_already_transmitted: Probability::ZERO,
            id: Uuid::new_v4(),
        };
        let b = Novelty {
            distance: 1.0,
            probability_already_transmitted: Probability::ZERO,
            id: Uuid::new_v4(),
        };
        dbg!(a.score());
        dbg!(b.score());
        assert!(a > b);
    }
}
