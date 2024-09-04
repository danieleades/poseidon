/// A probability value between 0 and 100%.
#[derive(Debug, Clone, Copy)]
pub struct Probability {
    /// 100% is represented by [`u32::MAX`].
    value: u32,
}

impl std::ops::Mul<Probability> for f64 {
    type Output = Self;

    fn mul(self, rhs: Probability) -> Self::Output {
        self * (Self::from(rhs.value) / Self::from(u32::MAX))
    }
}

impl Probability {
    pub const ONE_HUNDRED: Self = Self { value: u32::MAX };
    pub const ZERO: Self = Self { value: 0 };

    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { value }
    }

    #[must_use]
    pub const fn complement(self) -> Self {
        Self {
            value: u32::MAX - self.value,
        }
    }
}

impl TryFrom<f64> for Probability {
    type Error = ();

    /// Converts a f64 value to a Probability.
    ///
    /// The value must be between 0.0 and 100.0 (corresponding to 0% and 100%
    /// respectively).
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not between 0.0 and 100.0.
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (0.0..=100.0).contains(&value) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Self {
                value: (value * f64::from(u32::MAX) / 100.0) as u32,
            })
        } else {
            Err(())
        }
    }
}

impl From<Probability> for f64 {
    /// Converts a Probability to a f64.
    ///
    /// The returned value is between 0.0 and 100.0, where 100.0 corresponds to
    /// 100% probability.
    fn from(value: Probability) -> Self {
        Self::from(value.value) / Self::from(u32::MAX) * 100.0
    }
}

impl Ord for Probability {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Probability {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Probability {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Probability {}
