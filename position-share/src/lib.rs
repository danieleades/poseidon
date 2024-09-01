#![feature(impl_trait_in_assoc_type)]
use chrono::{DateTime, Utc};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use uuid::Uuid;

mod probability;

mod transmission_history;
use transmission_history::TransmissionHistory;

mod coordinate;
pub use coordinate::Coordinate;

mod novelty;
use novelty::Novelty;

type NodeId = Uuid;

/// A time-series collection of 3D coordinates.
///
/// Supports efficient filtering and searching by time.
#[derive(Debug, Clone, Default)]
pub struct Positions {
    transmission_history: TransmissionHistory,
    data: BTreeSet<Datum>,
}

/// A single data point in the time-series.
#[derive(Debug, Clone, PartialEq)]
pub struct Datum {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub coordinate: Coordinate,
}

impl Ord for Datum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for Datum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Datum {}

impl Positions {
    /// Adds a new position to the collection.
    ///
    /// TODO: This currently assumes that the timestamp is later than any previously added timestamp (without checking).
    pub fn add(&mut self, timestamp: DateTime<Utc>, position: Coordinate) -> Uuid {
        let id = Uuid::new_v4();
        self.data.insert(Datum {
            id,
            timestamp,
            coordinate: position,
        });
        id
    }

    /// Filters positions by a time range.
    pub fn filter_by_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> impl Iterator<Item = &Datum> {
        self.data
            .iter()
            .filter(move |datum| start <= datum.timestamp && datum.timestamp <= end)
    }

    /// Returns the most novel coordinates for a given recipient.
    ///
    /// The most novel coordinates are those that are furthest from the polyline generated by the already-transmitted (100% probability) coordinates, weighted by the probability that the recipient has not received them already.
    ///
    /// This method returns at most `n_max` results
    #[must_use]
    pub fn most_novel_coordinates(&self, recipient: &NodeId, n_max: usize) -> Vec<&Datum> {
        // First consider the first and last coordinates.
        let ((start, start_novelty), (end, end_novelty)) =
            self.start_and_end_point_novelty(recipient);
        let mut results = Results::new(n_max);
        results.insert(start, start_novelty);
        results.insert(end, end_novelty);

        // Then recursively search the rest of the coordinates.
        let segment = self.data.iter().collect::<Vec<_>>();
        let mut queue = VecDeque::new();
        queue.push_back(&segment[0..segment.len()]);

        while let Some(segment) = queue.pop_front() {
            if let Some((datum, novelty_score, index)) =
                self.most_novel_coordinate_in_segment(recipient, segment)
            {
                results.insert(datum, novelty_score);
                // Push the left and right subsegments onto the queue
                queue.push_back(&segment[1..=index]);
                queue.push_back(&segment[index..]);
            }
        }

        results.into_iter().collect()
    }

    /// Returns the novelty scores for the start and end coordinates.
    ///
    /// The novelty score is the distance between them, scaled by the probability that the recipient has not received them yet.
    fn start_and_end_point_novelty(
        &self,
        recipient: &NodeId,
    ) -> ((&Datum, Novelty), (&Datum, Novelty)) {
        let start = self.data.first().unwrap();
        let end = self.data.last().unwrap();
        let distance = (start.coordinate - end.coordinate).magnitude();

        let create_novelty = |datum: &Datum| Novelty {
            distance,
            probability_already_transmitted: self
                .transmission_history
                .probability_recipient_has_datum(recipient, &datum.id),
            id: datum.id,
        };

        let start_novelty = create_novelty(start);
        let end_novelty = create_novelty(end);

        ((start, start_novelty), (end, end_novelty))
    }

    /// Returns the most novel coordinate in a given segment, along with its novelty score and index.
    fn most_novel_coordinate_in_segment<'a>(
        &'a self,
        recipient: &NodeId,
        segment: &[&'a Datum],
    ) -> Option<(&'a Datum, Novelty, usize)> {
        // Algorithm:
        // 1. if there are less than 3 data points, return None
        // 2. find the most novel datum in the segment, excluding the first and last points
        // 3. return the most novel datum, its novelty score, and its index
        if segment.len() < 3 {
            return None;
        }

        let start = segment.first().unwrap();
        let end = segment.last().unwrap();

        segment[1..segment.len() - 1]
            .iter()
            .zip(1..)
            .map(|(datum, i)| {
                let distance =
                    distance_from_line(&start.coordinate, &end.coordinate, &datum.coordinate);
                let probability = self
                    .transmission_history
                    .probability_recipient_has_datum(recipient, &datum.id);
                let novelty = Novelty {
                    distance,
                    probability_already_transmitted: probability,
                    id: datum.id,
                };

                (*datum, novelty, i)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}

/// Calculates the perpendicular distance from a coordinate to a line defined by two coordinates.
fn distance_from_line(start: &Coordinate, end: &Coordinate, coordinate: &Coordinate) -> f64 {
    // Vector from start to end
    let line_vector = end - start;

    // Vector from start to the coordinate
    let point_vector = coordinate - start;

    // Calculate the cross product
    let cross_product = &line_vector.cross_product(&point_vector);

    // Calculate the magnitude of the cross product
    let cross_product_magnitude = cross_product.magnitude();

    // Calculate the magnitude of the line vector
    let line_magnitude = line_vector.magnitude();

    // The perpendicular distance is the magnitude of the cross product divided by the magnitude of the line vector
    cross_product_magnitude / line_magnitude
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

    /// Inserts a new datum into the results, keeping only the `n_max` most novel results.
    fn insert(&mut self, datum: &'a Datum, novelty: Novelty) {
        // There are less results than the maximum, so insert it with no further checks.
        if self.data.len() < self.n_max {
            self.data.insert(Reverse(novelty), datum);
        // The results are full, so only insert the datum if it is more novel than the least novel result.
        } else if let Some(min_novelty) = self.min_novelty() {
            if novelty > *min_novelty {
                self.data.pop_last();
                self.data.insert(Reverse(novelty), datum);
            }
        }
        // Only reachable if n_max is 0, meaning we don't want any results.
    }

    /// Returns the novelty score of the least novel coordinate in the results or 0.0 if the results are empty.
    fn min_novelty(&self) -> Option<&Novelty> {
        self.data
            .keys()
            .next_back()
            .map(|reverse_novelty| &reverse_novelty.0)
    }
}

impl<'a> IntoIterator for Results<'a> {
    type Item = &'a Datum;
    type IntoIter = impl Iterator<Item = Self::Item>;

    /// Returns an iterator over the results.
    ///
    /// Ordering: most novel to least novel
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::assert_approx_eq;

    #[test]
    fn test_distance_from_line() {
        let start = Coordinate::new(0.0, 0.0, 0.0);
        let end = Coordinate::new(1.0, 1.0, 1.0);
        let coordinate = Coordinate::new(0.5, 0.5, 0.5);
        assert_approx_eq!(f64, distance_from_line(&start, &end, &coordinate), 0.0);
    }

    #[test]
    fn test_distance_from_line2() {
        let start = Coordinate::new(0.0, 0.0, 0.0);
        let end = Coordinate::new(4.0, 0.0, 0.0);
        let coordinate = Coordinate::new(2.0, 2.0, 0.0);
        assert_approx_eq!(f64, distance_from_line(&start, &end, &coordinate), 2.0);
    }

    #[test]
    fn test_most_novel_coordinates() {
        // Coordinate arrangement:
        //
        //     y
        //     ^
        //   2 |           * (2,2)
        //     |
        //   1 |     * (1,1)     * (3,1)
        //     |
        //   0 * (0,0)                 * (4,0)
        //     +-----|-----|-----|-----|---->
        //     0     1     2     3     4     x

        let mut positions = Positions::default();
        let id0 = positions.add(Utc::now(), Coordinate::new(0.0, 0.0, 0.0));
        let _id1 = positions.add(Utc::now(), Coordinate::new(1.0, 1.0, 0.0));
        let id2 = positions.add(Utc::now(), Coordinate::new(2.0, 2.0, 0.0));
        let _id3 = positions.add(Utc::now(), Coordinate::new(3.0, 1.0, 0.0));
        let id4 = positions.add(Utc::now(), Coordinate::new(4.0, 0.0, 0.0));

        let most_novel = positions.most_novel_coordinates(&NodeId::new_v4(), 3);
        assert_eq!(most_novel.len(), 3);
        // Assert that the most novel positions contain the ids of points id0, id2, and id4
        let expected_ids = vec![id0, id2, id4];
        for expected_id in expected_ids {
            assert!(
                most_novel.iter().any(|datum| datum.id == expected_id),
                "Expected ID {expected_id} not found in most novel coordinates"
            );
        }
    }
}
