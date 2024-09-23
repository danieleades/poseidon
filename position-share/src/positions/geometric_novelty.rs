//! 'Geometric novelty' is a measure of how important a coordinate is in
//! describing the overall path.
//!
//! There are different algorithms for calculating geometric novelty, and this
//! crate provides a framework for plugging in different algorithms.
//!
//! An implementation of the [Ramer-Douglas-Peucker algorithm](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm) is provided.

use std::collections::BinaryHeap;

use crate::{positions::Datum, Coordinate};

/// A helper struct for sorting segments of the time-series by the most novel
/// coordinate in the segment.
///
/// This struct is a wrapper placed in a [`BinaryHeap`] in order to create a
/// max-heap.
#[derive(Debug, PartialEq)]
struct Comparator<'a, 'b> {
    pub segment: &'a [&'b Datum],
    pub datum: &'b Datum,
    pub distance: f64,
    pub index: usize,
}

impl<'a, 'b> Ord for Comparator<'a, 'b> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or_else(|| self.datum.id.cmp(&other.datum.id))
    }
}

impl<'a, 'b> PartialOrd for Comparator<'a, 'b> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, 'b> Eq for Comparator<'a, 'b> {}

/// A max-heap used to store segments of the time-series sorted by the most
/// geometrically novel coordinate in the segment.
#[derive(Debug, Default)]
pub struct MaxHeap<'a, 'b>(BinaryHeap<Comparator<'a, 'b>>);

impl<'a, 'b> MaxHeap<'a, 'b> {
    pub fn push(
        &mut self,
        segment: &'a [&'b Datum],
        datum: &'b Datum,
        distance: f64,
        index: usize,
    ) {
        self.0.push(Comparator {
            segment,
            datum,
            distance,
            index,
        });
    }

    pub fn pop(&mut self) -> Option<(&'a [&'b Datum], &'b Datum, f64, usize)> {
        self.0.pop().map(
            |Comparator {
                 segment,
                 datum,
                 distance,
                 index,
             }| (segment, datum, distance, index),
        )
    }
}

/// A trait for calculating the most novel coordinate in a segment of the
/// time-series.
pub trait GeometricNovelty {
    /// Calculates the most novel coordinate in a segment of the time-series.
    ///
    /// The first and last should be excluded. Only the interior points should
    /// be considered as candidates for the most novel coordinate.
    fn most_novel_coordinate<'a>(&self, segment: &[&'a Datum]) -> Option<(&'a Datum, f64, usize)>;
}

impl<F> GeometricNovelty for F
where
    F: for<'a> Fn(&[&'a Datum]) -> Option<(&'a Datum, f64, usize)>,
{
    fn most_novel_coordinate<'a>(&self, segment: &[&'a Datum]) -> Option<(&'a Datum, f64, usize)> {
        self(segment)
    }
}

/// A trait for calculating the novelty of a point in relation to its neighbors.
pub trait NoveltyMeasure {
    fn calculate_novelty(prev: &Coordinate, current: &Coordinate, next: &Coordinate) -> f64;
}

/// Perpendicular distance novelty measure (standard RDP)
pub struct PerpendicularDistance;

impl NoveltyMeasure for PerpendicularDistance {
    fn calculate_novelty(prev: &Coordinate, current: &Coordinate, next: &Coordinate) -> f64 {
        distance_from_line(prev, next, current)
    }
}

/// Area-based novelty measure
pub struct TriangleArea;

impl NoveltyMeasure for TriangleArea {
    fn calculate_novelty(prev: &Coordinate, current: &Coordinate, next: &Coordinate) -> f64 {
        triangle_area(prev, current, next)
    }
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
/// A 3D version of the [Ramer-Douglas-Peucker algorithm](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm) for calculating geometric novelty.
pub fn rdp_generic<'a, T: NoveltyMeasure>(
    segment: &[&'a Datum],
) -> Option<(&'a Datum, f64, usize)> {
    // Algorithm:
    // 1. if there are less than 3 data points, return None
    // 2. find the most novel datum in the segment, excluding the first and last
    //    points
    // 3. return the most novel datum, its novelty score, and its index
    if segment.len() < 3 {
        return None;
    }

    // These are safe to unwrap because we know the length of the segment is at
    // least 3
    #[allow(clippy::unwrap_used)]
    let start = segment.first().unwrap();
    #[allow(clippy::unwrap_used)]
    let end = segment.last().unwrap();

    segment[1..segment.len() - 1]
        .iter()
        .zip(1..)
        .map(|(datum, i)| {
            let distance =
                T::calculate_novelty(&start.coordinate, &end.coordinate, &datum.coordinate);
            (*datum, distance, i)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}

/// Standard RDP algorithm using perpendicular distance
#[must_use]
pub fn rdp<'a>(segment: &[&'a Datum]) -> Option<(&'a Datum, f64, usize)> {
    rdp_generic::<PerpendicularDistance>(segment)
}

/// Area-based RDP algorithm
#[must_use]
pub fn rdp_area<'a>(segment: &[&'a Datum]) -> Option<(&'a Datum, f64, usize)> {
    rdp_generic::<TriangleArea>(segment)
}

/// Calculates the perpendicular distance from a coordinate to a line defined by
/// two coordinates.
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

    // The perpendicular distance is the magnitude of the cross product divided by
    // the magnitude of the line vector
    cross_product_magnitude / line_magnitude
}

/// Calculates the area of a triangle formed by three 3D coordinates.
fn triangle_area(a: &Coordinate, b: &Coordinate, c: &Coordinate) -> f64 {
    let ab = b - a;
    let ac = c - a;
    let cross_product = ab.cross_product(&ac);
    0.5 * cross_product.magnitude()
}

#[cfg(test)]
mod tests {
    use float_cmp::assert_approx_eq;

    use super::*;

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
}
