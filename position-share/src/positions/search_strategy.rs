//! A search strategy is used to find the most novel coordinates in a
//! time-series.
//!
//! It should traverse the time-series and find the most novel coordinates
//! according to some heuristic.
//!
//! In general, search strategies may include:
//!  - recursively searching subsegments of the path
//!  - iteratively simplifying the path
//!
//! This module provides a framework for pluggable search strategies, and
//! provides a 'recursive search' strategy ([`Search`]).
//!
//! See [`rdp`](crate::positions::geometric_novelty::rdp) for an example of a
//! geometric novelty strategy which can be used with [`Search`].

use std::cmp::Ordering;
use std::{cmp::Reverse, collections::BTreeMap};

use super::{
    geometric_novelty::{GeometricNovelty, MaxHeap},
    Datum,
};
use crate::{probability::Probability, transmission_history::TransmissionHistory, NodeId};
use uuid::Uuid;

mod segment;

/// A search strategy for finding the most novel positions in a time-series.
pub trait SearchStrategy {
    fn search<'a>(
        &self,
        transmission_history: &TransmissionHistory,
        positions: Segment<'_, 'a>,
        n_max: usize,
        recipient: &NodeId,
    ) -> Vec<&'a Datum>;
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct Segment<'a, 'b> {
    /// The first point in the segment.
    start: &'b Datum,
    /// The middle points in the segment.
    /// 
    /// Guaranteed to be non-empty.
    middle: &'a [&'b Datum],
    /// The last point in the segment.
    end: &'b Datum,
}

impl<'a, 'b> Segment<'a, 'b> {
    pub const fn start(&self) -> &'b Datum {
        self.start
    }

    pub const fn middle(&self) -> &'a [&'b Datum] {
        self.middle
    }

    pub const fn end(&self) -> &'b Datum {
        self.end
    }

    pub const fn split_at(self, index: usize) -> (Option<Self>, Option<Self>) {
        if index == 0 {
            return (None, Some(self));
        }
        if index == self.middle.len() {
            return (Some(self), None);
        }
        let (left, right) = self.middle.split_at(index);
        (Some(Segment { start: self.start, middle: left, end: self.middle[index - 1] }), Some(Segment { start: self.middle[index], middle: right, end: self.end }))
    }
}

impl<'a, 'b> TryFrom<&'a [&'b Datum]> for Segment<'a, 'b> {
    type Error = ();

    /// Creates a new segment from a slice of positions.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the slice has less than 3 positions. This guarantees
    /// that the segment has at least one middle point.
    fn try_from(positions: &'a [&'b Datum]) -> Result<Self, Self::Error> {
        if positions.len() < 3 {
            return Err(());
        }
        Ok(Self {
            start: positions[0],
            middle: &positions[1..positions.len() - 1],
            end: positions[positions.len() - 1],
        })
    }
}
/// A search strategy which searches recursively through the time-series.
///
/// It first finds the most geometrically novel coordinate and then recursively
/// searches the left and right subsegments on either side of it.
///
/// The strategy for determining the most novel coordinate in a segment is
/// provided by the `GeometricNovelty` trait.
///
/// # Example
/// ```
/// use chrono::Utc;
/// use position_share::{rdp, Coordinate, NodeId, Positions, Search};
///
/// let mut positions = Positions::default();
/// positions.add(Utc::now(), Coordinate::new(0.0, 0.0, 0.0));
/// positions.add(Utc::now(), Coordinate::new(1.0, 1.0, 0.0));
/// positions.add(Utc::now(), Coordinate::new(2.0, 2.0, 0.0));
/// positions.add(Utc::now(), Coordinate::new(3.0, 1.0, 0.0));
/// positions.add(Utc::now(), Coordinate::new(4.0, 0.0, 0.0));
///
/// let search_strategy = Search::new(rdp, None);
/// let recipient = NodeId::new_v4();
///
/// let most_novel = positions.most_novel_coordinates(&search_strategy, &recipient, 3);
/// ```
pub struct Search<S>
where
    S: GeometricNovelty,
{
    strategy: S,
    threshold: Option<f64>,
}

impl<S> SearchStrategy for Search<S>
where
    S: GeometricNovelty,
{
    fn search<'a>(
        &self,
        transmission_history: &TransmissionHistory,
        positions: Segment<'_, 'a>,
        n_max: usize,
        recipient: &NodeId,
    ) -> Vec<&'a Datum> {
        // First consider the first and last coordinates.
        let (start_novelty, end_novelty) = start_and_end_point_novelty(positions.start(), positions.end());

        let mut results = Results::new(n_max);
        let first_datum = positions.start();
        let last_datum = positions.end();
        results.insert(
            first_datum,
            Novelty {
                distance: start_novelty,
                probability_not_transmitted: transmission_history
                    .probability_recipient_has_datum(recipient, &first_datum.id)
                    .complement(),
                id: first_datum.id,
            },
        );
        results.insert(
            last_datum,
            Novelty {
                distance: end_novelty,
                probability_not_transmitted: transmission_history
                    .probability_recipient_has_datum(recipient, &last_datum.id)
                    .complement(),
                id: last_datum.id,
            },
        );

        // Find the most novel coordinate in the first segment.
        let (datum, distance, index) = self.strategy.most_novel_coordinate(positions);
        let mut segment_heap = MaxHeap::default();
        segment_heap.push(positions, datum, distance, index);

        // Then search the rest of the coordinates.
        while let Some((segment, datum, distance, index)) = segment_heap.pop() {
            let novelty = Novelty {
                distance,
                probability_not_transmitted: transmission_history
                    .probability_recipient_has_datum(recipient, &datum.id)
                    .complement(),
                id: datum.id,
            };

            // stop condition
            if let (Some(min_novelty), Some(threshold)) = (results.min_novelty(), self.threshold) {
                if novelty < *min_novelty && distance < threshold * min_novelty.distance {
                    break;
                }
            }

            // Only insert the datum if the recipient has a non-zero probability of not
            // having received it yet.
            if novelty.probability_not_transmitted > Probability::ZERO {
                results.insert(datum, novelty);
            }
            // Push the left and right subsegments onto the queue

            let (left_segment, right_segment) = segment.split_at(index);

            for segment in [left_segment, right_segment].into_iter().flatten() {
                    let (datum, distance, index) = self.strategy.most_novel_coordinate(segment);
                    segment_heap.push(segment, datum, distance, index);
            }
        }
        results.into_iter().collect()
    }
}

impl<S> Search<S>
where
    S: GeometricNovelty,
{
    /// Create a new search strategy.
    ///
    /// If `threshold` is provided, the search stops when the geometric novelty
    /// of a subsegment is less than `threshold` times the geometric novelty of
    /// its parent segment.
    pub const fn new(strategy: S, threshold: Option<f64>) -> Self {
        Self {
            strategy,
            threshold,
        }
    }
}

/// Returns the geometric novelty scores for the start and end coordinates.
///
/// The novelty score is the distance between them
fn start_and_end_point_novelty(start: &Datum, end: &Datum) -> (f64, f64) {
    let distance = (start.coordinate - end.coordinate).magnitude();

    (distance, distance)
}

#[derive(Debug)]
struct Results<'a> {
    n_max: usize,
    data: BTreeMap<Reverse<Novelty>, &'a Datum>,
}

impl<'a> Results<'a> {
    /// Creates a new `Results` struct with a maximum of `n_max` results.
    const fn new(n_max: usize) -> Self {
        Self {
            n_max,
            data: BTreeMap::new(),
        }
    }

    /// Inserts a new datum into the results, keeping only the `n_max` most
    /// novel results.
    fn insert(&mut self, datum: &'a Datum, novelty: Novelty) {
        // There are less results than the maximum, so insert it with no further checks.
        if self.data.len() < self.n_max {
            self.data.insert(Reverse(novelty), datum);
        // The results are full, so only insert the datum if it is more novel
        // than the least novel result.
        } else if let Some(min_novelty) = self.min_novelty() {
            if novelty > *min_novelty {
                self.data.pop_last();
                self.data.insert(Reverse(novelty), datum);
            }
        }
        // Only reachable if n_max is 0, meaning we don't want any results.
    }

    /// Returns the novelty score of the least novel coordinate in the results
    /// or 0.0 if the results are empty.
    fn min_novelty(&self) -> Option<&Novelty> {
        self.data
            .keys()
            .next_back()
            .map(|reverse_novelty| &reverse_novelty.0)
    }
}

impl<'a> IntoIterator for Results<'a> {
    type IntoIter = std::collections::btree_map::IntoValues<Reverse<Novelty>, &'a Datum>;
    type Item = &'a Datum;

    /// Returns an iterator over the results.
    ///
    /// Ordering: most novel to least novel
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_values()
    }
}

#[derive(Debug, PartialEq)]
pub struct Novelty {
    pub distance: f64,
    pub probability_not_transmitted: Probability,
    pub id: Uuid,
}

impl Novelty {
    #[must_use]
    pub fn score(&self) -> f64 {
        self.distance * self.probability_not_transmitted
    }
}

impl Ord for Novelty {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score()
            .partial_cmp(&other.score())
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                self.probability_not_transmitted
                    .cmp(&other.probability_not_transmitted)
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
    use chrono::Utc;

    use crate::Coordinate;

    use super::*;

    #[test]
    fn compare() {
        let a = Novelty {
            distance: 2.0,
            probability_not_transmitted: Probability::ONE_HUNDRED,
            id: Uuid::new_v4(),
        };
        let b = Novelty {
            distance: 1.0,
            probability_not_transmitted: Probability::ONE_HUNDRED,
            id: Uuid::new_v4(),
        };
        assert!(a > b);
    }
}
