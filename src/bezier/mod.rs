use crate::{Evaluate, subdivide::Subdivide};

use super::vector::Vector;
use glifparser::{Point as GPPoint, PointData as GPPointData};

mod evaluate;
mod flo;

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct Bezier {
    pub w1: Vector,
    pub w2: Vector,
    pub w3: Vector,
    pub w4: Vector,
}

impl Bezier {
    // this function should accept lines, quadratic, and cubic segments and return a valid set of cubic bezier coefficients
    pub fn from<PD: GPPointData>(point: &GPPoint<PD>, next_point: &GPPoint<PD>) -> Self {
        let p = Vector::from_point(point);
        let np = Vector::from_point(next_point);
        let h1 = Vector::from_handle(point, point.a);
        let h2 = Vector::from_handle(next_point, next_point.b);

        return Self::from_points(p, h1, h2, np);
    }

    pub fn from_points(p0: Vector, p1: Vector, p2: Vector, p3: Vector) -> Self {
        return Bezier {
            w1: p0,
            w2: p1,
            w3: p2,
            w4: p3,
        };
    }

    pub fn to_control_points(&self) -> [Vector; 4] {
        [self.w1, self.w2, self.w3, self.w4]
    }

    pub fn to_control_points_vec(&self) -> Vec<Vector> {
        let controlps = self.to_control_points();

        let mut output = Vec::new();
        for p in &controlps {
            output.push(p.clone());
        }

        return output;
    }

    pub fn reverse(&self) -> Self {
        let bz = self.to_control_points();
        Bezier::from_points(bz[3], bz[2], bz[1], bz[0])
    }

    pub fn balance(&self) -> Bezier {
        let distance_heuristic = 0.1;

        let mut new_bez = self.clone();

        if self.w1.distance(self.w2) < distance_heuristic {
            new_bez.w2 = self.at(0.40);
        }

        if self.w3.distance(self.w4) < distance_heuristic {
            new_bez.w3 = self.at(0.60);
        }

        return new_bez;
    }
    
}

impl Subdivide for Bezier {
    fn split(&self, t: f64) -> Option<(Bezier, Bezier)> {
        if t == 1. || t == 0. { return None }

        // Perform De Casteljau's algorithm to split the curve at t
        let w12 = self.w1.lerp(self.w2, t);
        let w23 = self.w2.lerp(self.w3, t);
        let w34 = self.w3.lerp(self.w4, t);

        let w123 = w12.lerp(w23, t);
        let w234 = w23.lerp(w34, t);

        let w1234 = w123.lerp(w234, t);

        let first_half = Bezier::from_points(self.w1, w12, w123, w1234);
        let second_half = Bezier::from_points(w1234, w234, w34, self.w4);

        Some((first_half, second_half))
    }

}