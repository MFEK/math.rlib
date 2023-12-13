use crate::evaluate::Evaluate;
use crate::piecewise::Piecewise;
use crate::rect::Rect;
use crate::vector::Vector;

// Implements the evaluate trait for Piecewise
impl<T: Evaluate + Send + Sync> Evaluate for Piecewise<T> {
    // return the x, y of our curve at time t
    fn at(&self, t: f64) -> Vector {
        /*
        // there needs to be better handling than this probably through a fail/success
        if self.segs.len() == 0 {panic!("Can't evaluate an empty piecewise!")}

        // we multiply t by our segments then subtract the floored version of this value from the original to get
        // our offset t for that curve
        let modified_time = (self.segs.len()) as f64 * t;
        let curve_index = modified_time.floor().min((self.segs.len() - 1) as f64) as usize;
        let offset_time = modified_time - curve_index as f64;
        */

        let curve_index = self.seg_n(t);
        let offset_time = self.seg_t(t);

        let ref dir = self.segs[curve_index];

        return dir.at(offset_time);
    }

    // returns the derivative at time t
    fn tangent_at(&self, t: f64) -> Vector {
        /*
        // there needs to be better handling than this probably through a fail/success
        if self.segs.len() == 0 {panic!("Can't find derivative for an empty piecewise!")}

        // we multiply t by our segments then subtract the floored version of this value from the original to get
        // our offset t for that curve
        let modified_time = (self.segs.len()) as f64 * t;
        let curve_index = modified_time.floor().min((self.segs.len() - 1) as f64) as usize;
        let offset_time = modified_time - curve_index as f64;
        */

        let curve_index = self.seg_n(t);
        let offset_time = self.seg_t(t);

        let ref dir = self.segs[curve_index];

        return dir.tangent_at(offset_time);
    }

    fn bounds(&self) -> Rect {
        // again maybe success/failure? These are mainly here to catch bugs right now.
        if self.segs.len() == 0 {
            panic!("An empty piecewise knows no bounds!")
        }

        let mut output = Rect {
            left: f64::INFINITY,
            bottom: f64::INFINITY,
            right: -f64::INFINITY,
            top: -f64::INFINITY,
        };

        for curve in &self.segs {
            output = output.encapsulate_rect(curve.bounds());
        }

        return output;
    }

    fn apply_transform<F: Send + Sync>(&self, transform: F) -> Self
    where
        F: Fn(&Vector) -> Vector,
    {
        let output = self
            .segs
            .iter()
            .map(|contour| contour.apply_transform(&transform))
            .collect();

        return Piecewise::new(output, Some(self.cuts.clone()));
    }

    fn start_point(&self) -> Vector {
        if let Some(path_fcurve) = self.segs.first() {
            return path_fcurve.start_point();
        }

        // TODO: Add proper error handling to these functions.
        panic!("Empty piecewise has no start point.")
    }

    fn end_point(&self) -> Vector {
        if let Some(path_lcurve) = self.segs.last() {
            return path_lcurve.end_point();
        }

        // TODO: Add proper error handling to these functions.
        panic!("Empty piecewise has no start point.")
    }
}
