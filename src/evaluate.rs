use super::coordinate::Coordinate;
use super::rect::Rect;
use super::vector::Vector;

use crate::vec2;

// Any object in a piecewise MUST implement this trait! This trait essentially says that our struct
// can be evaluated with respect to time t and return an x, y pair. It also needs to be able to give us
// a derivative and a bounding box.
// Could probably use a better name. Maybe Primitive as they're the building blocks of our glyph.
pub trait Evaluate {
    type EvalResult: Coordinate + Send + Sync;
    fn at(&self, t: f64) -> Self::EvalResult;
    fn tangent_at(&self, u: f64) -> Self::EvalResult;
    fn bounds(&self) -> Rect; // returns an AABB that contains all points
    fn apply_transform<F: Send + Sync>(&self, transform: F) -> Self
    where
        F: Fn(&Self::EvalResult) -> Self::EvalResult;
    fn start_point(&self) -> Self::EvalResult;
    fn end_point(&self) -> Self::EvalResult;
}

pub trait EvalTranslate: Evaluate {
    fn translate(&self, t: Self::EvalResult) -> Self;
}

pub trait EvalScale: Evaluate {
    fn scale(&self, s: Self::EvalResult) -> Self;
}

pub trait EvalRotate: Evaluate {
    fn rotate(&self, angle: f64) -> Self;
}

impl<T: Evaluate + Send + Sync> EvalTranslate for T {
    fn translate(&self, t: T::EvalResult) -> Self {
        let transform = |v: &T::EvalResult| *v + t;

        self.apply_transform(&transform)
    }
}

impl<T: Evaluate> EvalScale for T {
    fn scale(&self, s: T::EvalResult) -> Self {
        let transform = |v: &T::EvalResult| *v * s;

        self.apply_transform(&transform)
    }
}

impl<T: Evaluate<EvalResult = Vector>> EvalRotate for T {
    fn rotate(&self, angle: f64) -> Self {
        let transform = |v: &T::EvalResult| {
            return vec2!(
                v.x * f64::cos(angle) - v.y * f64::sin(angle),
                v.x * f64::sin(angle) + v.y * f64::cos(angle)
            );
        };

        self.apply_transform(&transform)
    }
}
