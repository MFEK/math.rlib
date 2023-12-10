use flo_curves::bezier::{de_casteljau3, derivative3, de_casteljau2};
use glifparser::{glif::point::quad::QPoint, PointData};

use crate::{Vector, Rect, Evaluate, subdivide::Subdivide};

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct QuadBezier {
    pub w1: Vector,
    pub w2: Vector,
    pub w3: Vector,
}

impl QuadBezier {
    // this function should accept lines, quadratic, and cubic segments and return a valid set of cubic bezier coefficients
    pub fn from<PD: PointData>(point: &QPoint<PD>, next_point: &QPoint<PD>) -> Self {
        let p = Vector::from_quad_point(point);
        let np = Vector::from_quad_point(next_point);
        let h1 = Vector::from_quad_handle(point);

        return Self::from_points(p, h1, np);
    }

    pub fn from_points(p0: Vector, p1: Vector, p2: Vector) -> Self {
        return QuadBezier {
            w1: p0,
            w2: p1,
            w3: p2,
        };
    }

    pub fn to_control_points(&self) -> [Vector; 3] {
        [self.w1.clone(), self.w2.clone(), self.w3.clone()]
    }
    
    pub fn calc_line_intersection(&self, line_start: Vector, line_end: Vector) -> Vec<Vector> {
        let mut intersections = Vec::new();

        // Inverse line normal
        let normal = Vector { x: line_start.y - line_end.y, y: line_end.x - line_start.x };

        // Q-coefficients
        let c2 = Vector {
            x: self.w1.x + self.w2.x * -2.0 + self.w3.x,
            y: self.w1.y + self.w2.y * -2.0 + self.w3.y,
        };

        let c1 = Vector {
            x: self.w1.x * -2.0 + self.w2.x * 2.0,
            y: self.w1.y * -2.0 + self.w2.y * 2.0,
        };

        let c0 = self.w1;

        // Transform to line
        let coefficient = line_start.x * line_end.y - line_end.x * line_start.y;
        let a = normal.x * c2.x + normal.y * c2.y;
        let b = (normal.x * c1.x + normal.y * c1.y) / a;
        let c = (normal.x * c0.x + normal.y * c0.y + coefficient) / a;

        // Solve the roots
        let mut roots = Vec::new();
        let d = b * b - 4.0 * c;
        if d > 0.0 {
            let e = f64::sqrt(d);
            roots.push((-b + f64::sqrt(d)) / 2.0);
            roots.push((-b - f64::sqrt(d)) / 2.0);
        } else if d == 0.0 {
            roots.push(-b / 2.0);
        }

        // Calculate the solution points
        for t in roots {
            let minX = line_start.x.min(line_end.x);
            let minY = line_start.y.min(line_end.y);
            let maxX = line_start.x.max(line_end.x);
            let maxY = line_start.y.max(line_end.y);

            if t >= 0.0 && t <= 1.0 {
                // Possible point -- pending bounds check
                let point = Vector {
                    x: self.at(t).x,
                    y: self.at(t).y,
                };

                let x = point.x;
                let y = point.y;

                // Bounds checks
                if line_start.x == line_end.x && y >= minY && y <= maxY {
                    // Vertical line
                    intersections.push(point);
                } else if line_start.y == line_end.y && x >= minX && x <= maxX {
                    // Horizontal line
                    intersections.push(point);
                } else if x >= minX && y >= minY && x <= maxX && y <= maxY {
                    // Line passed bounds check
                    intersections.push(point);
                }
            }
        }

        intersections
    }
}

impl Evaluate for QuadBezier {
    fn at(&self, t: f64) -> Vector {
        let d1 = self.w1.clone();
        let d2 = self.w2.clone();
        let d3 = self.w3.clone();

        de_casteljau3(t, d1, d2, d3)
    }

    fn tangent_at(&self, t: f64) -> Vector {
        // Extract the points that make up this curve
        let w1 = self.w1.clone();
        let w2 = self.w2.clone();
        let w3 = self.w3.clone();

        // If w1 == w2 or w2 == w3, there will be an anomaly at t=0.0 and t=1.0
        // (it's probably mathematically correct to say there's no tangent at these points,
        // but the result is surprising and probably useless in a practical sense)
        let t = if t == 0.0 { f64::EPSILON } else { t };
        let t = if t == 1.0 { 1.0 - f64::EPSILON } else { t };

        // Get the deriviative
        let (d1, d2) = derivative3(w1, w2, w3);

        // Get the tangent and the point at the specified t value
        let tangent = de_casteljau2(t, d1, d2);

        tangent
    }

    fn apply_transform<F>(&self, transform: F) -> Self
    where
        F: Fn(&Vector) -> Vector,
    {
        let tp: [Vector; 3] = [
            transform(&self.w1),
            transform(&self.w2),
            transform(&self.w3),
        ];

        QuadBezier {
            w1: tp[0].clone(),
            w2: tp[1].clone(),
            w3: tp[2].clone(),
        }
    }

    fn bounds(&self) -> Rect {
        // Assuming to_control_points_vec is available in your code
        Rect::AABB_from_points(vec![self.w1.clone(), self.w2.clone(), self.w3.clone()])
    }

    fn start_point(&self) -> Vector {
        self.w1.clone()
    }

    fn end_point(&self) -> Vector {
        self.w3.clone()
    }
}

impl Subdivide for QuadBezier {
    fn split(&self, t: f64) -> Option<(QuadBezier, QuadBezier)> {
        if t == 1. || t == 0. {
            return None;
        }

        // Perform De Casteljau's algorithm to split the curve at t
        let q1 = self.w1.lerp(self.w2, t);
        let q2 = self.w2.lerp(self.w3, t);

        let r1 = q1.lerp(q2, t);

        let first_half = QuadBezier::from_points(self.w1.clone(), q1, r1);
        let second_half = QuadBezier::from_points(r1, q2, self.w3.clone());

        Some((first_half, second_half))
    }
}