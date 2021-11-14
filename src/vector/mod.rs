mod conv;
mod skia;

pub mod glif;
pub mod flo;

use super::coordinate::Coordinate;

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

#[macro_export]
macro_rules! vec2 {
    ($x: expr, $y: expr) => {
        Vector {x: $x as f64, y: $y as f64} // these f64 don't hurt if passed value already is, yet handle f32
    };
}

impl Vector {
    #[cfg(features = "strict")]
    pub fn from_components(x: f64, y: f64) -> Self
    {
        assert!(!x.is_nan());
        assert!(!y.is_nan());
        assert!(x.is_finite());
        assert!(y.is_finite());
        Vector { x, y }
    }

    #[cfg(not(features = "strict"))]
    pub fn from_components(x: f64, y: f64) -> Self
    {
        Vector { x, y }
    }

    pub fn is_near(self, v1: Vector, eps: f64) -> bool
    {
        self.x - v1.x <= eps && self.x - v1.x >= -eps &&
        self.y - v1.y <= eps && self.y - v1.y >= -eps
    }

    pub fn magnitude(self) -> f64
    {
        f64::sqrt(f64::powi(self.x, 2) + f64::powi(self.y, 2))
    }
    
    pub fn distance(self, v1: Vector) -> f64
    {
        let v0 = self;
        f64::sqrt(f64::powi(v1.x - v0.x, 2) + f64::powi(v1.y - v0.y, 2))
    }

    pub fn normalize(self) -> Self
    {
        let magnitude = self.magnitude();
        Vector { x: self.x / magnitude, y: self.y / magnitude }
    }

    pub fn dot(self, v1: Vector) -> f64
    {
        self.x * v1.x + self.y * v1.y
    }

    pub fn lerp(self, v1:Vector, t: f64) -> Self
    {
        let v0 = self;
        Vector {
            x: (1. - t) * v0.x + t * v1.x,
            y: (1. - t) * v0.y + t * v1.y
        }
    }

    pub fn angle(self, v1: Vector) -> f64 {
        return f64::atan2(v1.y, v1.x) - f64::atan2(self.y, self.x);
    }

    pub fn rotate(self, pivot: Vector, angle: f64) -> Vector {
        let s = f64::sin(angle);
        let c = f64::cos(angle);

        let translated_point = self - pivot;
        let rotated_point = Vector {
            x: translated_point.x * c - translated_point.y * s,
            y: translated_point.x * s + translated_point.y * c,
        };

        return rotated_point + pivot;
    }
}

impl Coordinate for Vector {
    fn magnitude(self) -> f64 
    {
        self.magnitude()
    }

    fn distance(self, v1: Self) -> f64
    {
        self.distance(v1)
    }

    fn lerp(self, v1: Self, t: f64) -> Self
    {
        self.lerp(v1, t)
    }
}
