use super::rect::Rect;
use super::vector::Vector;

use crate::vec2;

// Any object in a piecewise MUST implement this trait! This trait essentially says that our struct
// can be evaluated with respect to time t and return an x, y pair. It also needs to be able to give us
// a derivative and a bounding box.
// Could probably use a better name. Maybe Primitive as they're the building blocks of our glyph.
pub trait Evaluate {
    fn at(&self, t: f64) -> Vector;
    fn tangent_at(&self, u: f64) -> Vector;
    fn bounds(&self) -> Rect; // returns an AABB that contains all points
    fn apply_transform<F: Send + Sync>(&self, transform: F) -> Self
    where
        F: Fn(&Vector) -> Vector;
    fn start_point(&self) -> Vector;
    fn end_point(&self) -> Vector;
}

pub trait EvalTranslate: Evaluate {
    fn translate(&self, t: Vector) -> Self;
}

pub trait EvalScale: Evaluate {
    fn scale(&self, s: Vector) -> Self;
}

pub trait EvalRotate: Evaluate {
    fn rotate(&self, angle: f64) -> Self;
}

impl<T: Evaluate + Send + Sync> EvalTranslate for T {
    fn translate(&self, t: Vector) -> Self {
        let transform = |v: &Vector| {
            return *v + t;
        };

        return self.apply_transform(&transform);
    }
}

impl<T: Evaluate> EvalScale for T {
    fn scale(&self, s: Vector) -> Self {
        let transform = |v: &Vector| {
            return *v * s;
        };

        return self.apply_transform(&transform);
    }
}

impl<T: Evaluate> EvalRotate for T {
    fn rotate(&self, angle: f64) -> Self {
        let transform = |v: &Vector| {
            return vec2!(
                v.x * f64::cos(angle) - v.y * f64::sin(angle),
                v.x * f64::sin(angle) + v.y * f64::cos(angle)
            );
        };

        return self.apply_transform(&transform);
    }
}
