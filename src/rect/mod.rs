#[cfg(feature="default")]
mod flip_if_required;
#[cfg(feature="default")]
pub use flip_if_required::FlipIfRequired;

use super::vector::Vector;

// An axis-aligned rectangle. Sort of a stub right now to make some function outputs more legible.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64
}

impl Rect {
    // We expand the rect to be the minimum axis aligned bounding box that holds both our current rect and the point.
    pub fn encapsulate(&self, p: Vector) -> Rect
    {
        let mut lx = self.left;
        let mut ly = self.bottom;
        let mut hx = self.right;
        let mut hy = self.top;
    
        if p.x > hx { hx = p.x }
        if p.y > hy { hy = p.y }
        if p.x < lx { lx = p.x }
        if p.y < ly { ly = p.y }
    
        return Rect {
            left: lx,
            right: hx,
            top: hy,
            bottom: ly
        };
    }

    // Taje a vec of points and return a minimum bounding box for that point.
    #[allow(non_snake_case)]
    pub fn AABB_from_points(points: Vec<Vector>) -> Self
    {
        let mut lx = f64::INFINITY;
        let mut ly = f64::INFINITY;
        let mut hx = -f64::INFINITY;
        let mut hy = -f64::INFINITY;
    
        for p in points {
            if p.x > hx { hx = p.x }
            if p.y > hy { hy = p.y }
            if p.x < lx { lx = p.x }
            if p.y < ly { ly = p.y }
        }
    
        return Rect {
            left: lx,
            right: hx,
            top: hy,
            bottom: ly
        };
    }

    pub fn area(&self) -> f64 {
        (self.right - self.left) *  (self.top - self.bottom)
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        if  self.bottom < other.top &&
            self.top > other.bottom &&
            self.left < other.right &&
            self.right > other.left {
               return true;
        }
        false
    }

    // returns the resulting AABB from two other Rect's overlap
    pub fn overlap_rect(&self, other: &Rect) -> Rect {
        Rect {
            left: self.left.max(other.left),
            bottom: self.bottom.max(other.bottom),
            right: self.right.min(other.right),
            top: self.top.min(other.top),
        }
    }

    pub fn encapsulate_rect(&self, other: Rect) -> Rect
    {
        let left_bottom = Vector{x: other.left, y: other.bottom};
        let right_top = Vector{x: other.right, y: other.top};
        return self.encapsulate(left_bottom).encapsulate(right_top)
    }

    pub fn width(&self) -> f64 {
        f64::abs(self.left - self.right)
    }

    pub fn height(&self) -> f64 {
        f64::abs(self.top - self.bottom)
    }

    pub fn center(&self) -> Vector {
        let left_bottom = Vector::from_components(self.left, self.bottom);
        let right_top = Vector::from_components(self.right, self.top);

        return left_bottom.lerp(right_top, 0.5);
    }
}
