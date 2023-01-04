use super::super::{Vector, Evaluate, Rect, Bezier};

use flo_curves::BezierCurve;

use flo_curves::bezier::{derivative4, de_casteljau3, de_casteljau4};

impl Evaluate for Bezier {
    type EvalResult = Vector;
    
    fn at(&self, t: f64) -> Vector
    {
        let d1 = BezierCurve::start_point(self);
        let (d2, d3) = BezierCurve::control_points(self);
        let d4 = BezierCurve::end_point(self);

        de_casteljau4(t, d1, d2, d3, d4)
    }

    
    fn tangent_at(&self, t: f64) -> Vector
    {
            // Extract the points that make up this curve
            let w1          = BezierCurve::start_point(self);
            let (w2, w3)    = BezierCurve::control_points(self);
            let w4          = BezierCurve::end_point(self);
    
            // If w1 == w2 or w3 == w4 there will be an anomaly at t=0.0 and t=1.0 
            // (it's probably mathematically correct to say there's no tangent at these points but the result is surprising and probably useless in a practical sense)
            let t = if t == 0.0 { f64::EPSILON }        else { t };
            let t = if t == 1.0 { 1.0-f64::EPSILON }    else { t };
    
            // Get the deriviative
            let (d1, d2, d3) = derivative4(w1, w2, w3, w4);
    
            // Get the tangent and the point at the specified t value
            let tangent     = de_casteljau3(t, d1, d2, d3);
    
            tangent
    }

    fn apply_transform<F>(&self, transform: F) -> Self where F: Fn(&Vector) -> Vector
    {
        let original_points = self.to_control_points();
        let tp: [Vector; 4] = [
            transform(&original_points[0]),
            transform(&original_points[1]),
            transform(&original_points[2]),
            transform(&original_points[3]),
        ];

        return Bezier::from_points(tp[0], tp[1], tp[2], tp[3]);
    }

    fn bounds(&self) -> Rect
    {
        return Rect::AABB_from_points(self.to_control_points_vec());
    }

    fn start_point(&self) -> Vector
    {
        return self.to_control_points()[0];

    }

    fn end_point(&self) -> Vector
    {
        return self.to_control_points()[3];
    }

}

impl Bezier {
    /// Returns the second derivative of the Bezier curve at time `t`.
    ///
    /// The second derivative of a curve at a particular point is a measure of how the curve is
    /// changing at that point. It is the derivative of the derivative of the curve. A positive
    /// second derivative indicates that the curve is concave up (curving upwards) at that point,
    /// while a negative second derivative indicates that the curve is concave down (curving
    /// downwards) at that point.
    pub fn second_derivative_at(&self, t: f64) -> (f64, f64) {
        let tan = self.tangent_at(t);
        let (x_derivative, y_derivative) = (tan.x, tan.y);

        macro_rules! second_derivative {
            ($t:expr, $first_derivative:expr, $control_point:expr, $end_point:expr) => {
                $first_derivative * 3.0 * (1.0 - $t) * (1.0 - $t) +
                    6.0 * (1.0 - $t) * $t * $control_point +
                    3.0 * $t * $t * $end_point
            }
        }

        let x_second_derivative = second_derivative!(t, x_derivative, self.w2.x, self.w3.x);
        let y_second_derivative = second_derivative!(t, y_derivative, self.w2.y, self.w3.y);

        return (x_second_derivative, y_second_derivative);
    }

}

impl Bezier {
    pub fn min_curvature(&self) -> f64 {
        let numerator = 3.0 * self.w2.x - self.w1.x;
        let denominator = 6.0 * self.w2.x - 2.0 * self.w1.x - 3.0 * self.w3.x;
        return numerator / denominator;
    }

    pub fn max_curvature(&self) -> f64 {
        let numerator = 3.0 * self.w3.x - self.w4.x;
        let denominator = 3.0 * self.w3.x - self.w4.x + 2.0 * self.w2.x - self.w1.x;
        return numerator / denominator;
    }
}

