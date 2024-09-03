use std::collections::BinaryHeap;

use crate::positions::Datum;

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
