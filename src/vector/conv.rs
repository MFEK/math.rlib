/// Conversion boilerplate
use std::ops::{Add, Mul, Div, Neg, Sub};
use std::ops::{AddAssign, MulAssign, DivAssign, SubAssign};
use std::ops::{Index, IndexMut};

use super::Vector;

impl From<(f64, f64)> for Vector {
    fn from((x, y): (f64, f64)) -> Vector {
        Vector::from_components(x, y)
    }
}

impl From<(f32, f32)> for Vector {
    fn from((x, y): (f32, f32)) -> Vector {
        Vector::from_components(x as f64, y as f64)
    }
}

impl From<Vector> for (f64, f64) {
    fn from(v: Vector) -> Self {
        (v.x, v.y)
    }
}

impl From<Vector> for (f32, f32) {
    fn from(v: Vector) -> Self {
        (v.x as f32, v.y as f32)
    }
}

impl From<&[f64]> for Vector {
    fn from(s: &[f64]) -> Vector {
        debug_assert_eq!(s.len(), 2);
        Vector::from_components(s[0], s[1])
    }
}

impl From<&[f32]> for Vector {
    fn from(s: &[f32]) -> Vector {
        debug_assert_eq!(s.len(), 2);
        Vector::from_components(s[0] as f64, s[1] as f64)
    }
}

impl From<[f64; 2]> for Vector {
    fn from(s: [f64; 2]) -> Vector {
        Vector::from_components(s[0], s[1])
    }
}

impl From<[f32; 2]> for Vector {
    fn from(s: [f32; 2]) -> Vector {
        Vector::from_components(s[0] as f64, s[1] as f64)
    }
}

impl From<f64> for Vector {
    fn from(f: f64) -> Vector {
        Vector::from_components(f, f)
    }
}

impl From<f32> for Vector {
    fn from(f: f32) -> Vector {
        (f as f64).into()
    }
}

impl std::cmp::PartialEq for Vector {
    fn eq(&self, other: &Vector) -> bool {
        return self.x == other.x && self.y == other.y;
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;
    fn add(self, other: Vector) -> Vector {
        Vector::from_components(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign<Vector> for Vector {
    fn add_assign(&mut self, other: Vector) {
        *self = Vector::from_components(self.x + other.x, self.y + other.y);
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;
    fn sub(self, other: Vector) -> Vector {
        Vector::from_components(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign<Vector> for Vector {
    fn sub_assign(&mut self, other: Vector) {
        *self = Vector::from_components(self.x - other.x, self.y - other.y);
    }
}

impl Mul<Vector> for Vector {
    type Output = Vector;
    fn mul(self, other: Vector) -> Vector {
        Vector::from_components(self.x * other.x, self.y * other.y)
    }
}

impl MulAssign<Vector> for Vector {
    fn mul_assign(&mut self, other: Vector) {
        *self = Vector::from_components(self.x * other.x, self.y * other.y);
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, s: f64) -> Vector {
        Vector::from_components(self.x * s, self.y * s)
    }
}

/// TODO: Define these for everything else.
impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, other: Vector) -> Self::Output {
        Vector::from_components(self * other.x, self * other.y)
    }
}

impl MulAssign<f64> for Vector {
    fn mul_assign(&mut self, s: f64) {
        *self = Vector::from_components(self.x * s, self.y * s);
    }
}

impl Div<Vector> for Vector {
    type Output = Vector;
    fn div(self, other: Vector) -> Vector {
        Vector::from_components(self.x / other.x, self.y / other.y)
    }
}

impl DivAssign<f64> for Vector {
    fn div_assign(&mut self, s: f64) {
        *self = Vector::from_components(self.x / s, self.y / s);
    }
}

impl Div<f64> for Vector {
    type Output = Vector;
    fn div(self, s: f64) -> Vector {
        Vector::from_components(self.x / s, self.y / s)
    }
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector::from_components(-self.x, -self.y)
    }
}

impl Index<usize> for Vector {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("can only index Vector by 0 or 1")
        }
    }
}

impl IndexMut<usize> for Vector {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("can only index Vector by 0 or 1")
        }
    }
}
