use crate::Evaluate;

use super::vector::Vector;
use glifparser::{Point as GPPoint, PointData as GPPointData, WhichHandle};

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
        let h1 = Vector::from_handle(point, WhichHandle::A);
        let h2 = Vector::from_handle(next_point, WhichHandle::B);

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

    pub fn split_at_multiple_t(&self, mut t_values: Vec<f64>) -> Vec<Bezier> {
        // Sort the t-values to make it easier to split the curve
        t_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
        let mut beziers: Vec<Bezier> = Vec::new();
        let mut last_t: f64 = 0.0;
    
        // Store the current segment; initially, it's the whole Bezier curve
        let mut current_bezier = self.clone();
    
        for &t in &t_values {
            // Normalize t to the remaining part of the curve
            let local_t = (t - last_t) / (1.0 - last_t);
    
            // Perform the split
            let (left, right) = current_bezier.split(local_t);
    
            // Store the left (first) part of the split
            beziers.push(left);
    
            // Update the current segment to be the right (second) part of the split
            current_bezier = right;
    
            // Update the last t-value
            last_t = t;
        }
    
        // Add the remaining part of the curve
        beziers.push(current_bezier);
    
        beziers
    }

    pub fn split(&self, t: f64) -> (Bezier, Bezier) {
        // Perform De Casteljau's algorithm to split the curve at t
        let w12 = self.w1.lerp(self.w2, t);
        let w23 = self.w2.lerp(self.w3, t);
        let w34 = self.w3.lerp(self.w4, t);

        let w123 = w12.lerp(w23, t);
        let w234 = w23.lerp(w34, t);

        let w1234 = w123.lerp(w234, t);

        let first_half = Bezier::from_points(self.w1, w12, w123, w1234);
        let second_half = Bezier::from_points(w1234, w234, w34, self.w4);

        (first_half, second_half)
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
