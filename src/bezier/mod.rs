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
}
