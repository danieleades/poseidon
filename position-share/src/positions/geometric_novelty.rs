use std::collections::BinaryHeap;

use crate::{positions::Datum, Coordinate};

/// A helper struct for sorting segments of the time-series by the most novel coordinate in the segment.
///
/// This struct is a wrapper placed in a [`BinaryHeap`] in order to create a max-heap.
#[derive(Debug, PartialEq)]
pub struct Comparator<'a, 'b> {
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

/// A max-heap used to store segments of the time-series sorted by the most geometrically novel coordinate in the segment.
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

pub trait GeometricNovelty {
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

pub fn rdp<'a>(segment: &[&'a Datum]) -> Option<(&'a Datum, f64, usize)> {
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
            (*datum, distance, i)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
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

fn most_novel_coordinate_in_segment_with_strategy<'a>(
    strategy: &impl GeometricNovelty,
    segment: &[&'a Datum],
) -> Option<(&'a Datum, f64, usize)> {
    strategy.most_novel_coordinate(segment)
}

fn most_novel_coordinate_in_segment_with_rdp<'a>(
    segment: &[&'a Datum],
) -> Option<(&'a Datum, f64, usize)> {
    most_novel_coordinate_in_segment_with_strategy(&rdp, segment)
}
