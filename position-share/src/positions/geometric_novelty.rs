use crate::positions::Datum;

/// A helper struct for sorting segments of the time-series by the most novel coordinate in the segment.
///
/// This struct is a wrapper placed in a [`BinaryHeap`] in order to create a max-heap.
pub struct Comparator<'a, 'b> {
    pub segment: &'a [&'b Datum],
    pub datum: &'b Datum,
    pub distance: f64,
    pub index: usize,
}
