/// Represents a 3D coordinate.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Coordinate {
    /// Creates a new Coordinate.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl std::ops::Sub for Coordinate {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Sub for &Coordinate {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[allow(clippy::suboptimal_flops)] // benchmarking shows this is actually faster
    pub fn cross_product(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[allow(clippy::suboptimal_flops)] // benchmarking shows this is actually faster
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::assert_approx_eq;

    use super::*;

    #[test]
    fn test_cross_product() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(4.0, 5.0, 6.0);
        let expected = Vector::new(-3.0, 6.0, -3.0);
        assert_eq!(v1.cross_product(&v2), expected);
    }

    #[test]
    fn test_magnitude() {
        let v = Vector::new(3.0, 4.0, 0.0);
        assert_approx_eq!(f64, v.magnitude(), 5.0);
    }
}
