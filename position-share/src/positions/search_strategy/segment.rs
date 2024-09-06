#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct Segment<'a, 'b, T> {
    data: &'a [&'b T],
}

impl<'a, 'b, T> Segment<'a, 'b, T> {
    pub const fn start(&self) -> &'b T {
        self.data[0]
    }

    pub fn middle(&self) -> &'a [&'b T] {
        &self.data[1..self.data.len() - 1]
    }

    pub const fn end(&self) -> &'b T {
        self.data[self.data.len() - 1]
    }

    /// Split the segment at the given index.
    /// 
    /// The index is the index within the middle slice (excluding the start and end).
    /// 
    /// If the resulting subsegment's middle slice would be empty, None is returned for that subsegment.
    /// 
    /// # Panics
    /// 
    /// Panics if the index is out of bounds.
    pub fn split_at(self, index: usize) -> (Option<Self>, Option<Self>) {
        let left = &self.data[..=index];
        let right = &self.data[index..];
        (Self::try_from(left).ok(), Self::try_from(right).ok())
    }
}

impl<'a, 'b, T> TryFrom<&'a [&'b T]> for Segment<'a, 'b, T> {
    type Error = ();

    /// Creates a new segment from a slice of positions.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the slice has less than 3 positions. This guarantees
    /// that the segment has at least one middle point.
    fn try_from(positions: &'a [&'b T]) -> Result<Self, Self::Error> {
        if positions.len() < 3 {
            return Err(());
        }
        Ok(Self { data: positions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_split_at() {
        let data = [1, 2, 3, 4, 5, 6];
        let positions = data.iter().collect::<Vec<_>>();
        let segment = Segment::try_from(&positions[..]).unwrap();

        let (left, right) = segment.split_at(2);
        assert_eq!(left, Some(Segment::try_from(&positions[..3]).unwrap()));
        assert_eq!(right, Some(Segment::try_from(&positions[2..]).unwrap()));
    }
}