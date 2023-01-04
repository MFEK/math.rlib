mod evaluate;
pub mod glif;
#[cfg(feature = "skia")]
mod skia;

use crate::consts::SMALL_DISTANCE;

use crate::arclenparameterization::ArcLengthParameterization;
use crate::bezier::Bezier;
use crate::evaluate::Evaluate;
use crate::primitive::Primitive;
use crate::vector::Vector;

use itertools::Itertools;

// This struct models a simple piecewise function. It maps 0-1 such that 0 is the beginning of the first curve
// in the collection and 1 is the end of the last.
#[derive(Clone, Debug, Default)]
pub struct Piecewise<T: Evaluate> {
    pub cuts: Vec<f64>,
    // this should definitely change to private at some point with an iterator or getter to access
    pub segs: Vec<T>,
}

impl<T: Evaluate> Piecewise<T> {
    pub fn new(segs: Vec<T>, cuts: Option<Vec<f64>>) -> Self {
        match cuts {
            Some(cuts) => return Self { cuts, segs },

            // if we are given just a list of segments we generate the cuts ourselves
            _ => {
                let mut out_cuts: Vec<f64> = Vec::new();

                out_cuts.push(0.);

                let seg_iter = segs.iter().enumerate().peekable();
                let seg_len = segs.len();

                for (i, _seg) in seg_iter {
                    out_cuts.push((i + 1) as f64 / seg_len as f64);
                }

                return Self {
                    cuts: out_cuts,
                    segs,
                };
            }
        }
    }

    // implementation ripped from lib2geom, performs a binary search to find our segment
    pub fn seg_n(&self, t: f64) -> usize {
        let mut left = 0;
        let mut right = self.cuts.len() - 1;

        while left < right {
            let middle = (right + left) / 2;

            if left == middle {
                return middle;
            }
            if right == middle {
                return left;
            }
            if self.cuts[middle] == t {
                return middle;
            };

            if self.cuts[middle] < t {
                left = middle
            } else {
                right = middle;
            }
        }

        // This needs to be replaced with success/failure.
        panic!("Couldn't find the target segment!");
    }

    pub fn seg_t(&self, t: f64) -> f64 {
        let i = self.seg_n(t);
        return (t - self.cuts[i]) / (self.cuts[i + 1] - self.cuts[i]);
    }
}

// TODO: Move these functions to a more appropriate submodule.
impl<T: Evaluate<EvalResult = Vector> + Primitive + Send + Sync> Piecewise<Piecewise<T>> {
    // we split the primitive that contains t at t
    pub fn subdivide(&self, t: f64) -> Self {
        let mut output = Vec::new();
        for contour in &self.segs {
            output.push(contour.subdivide(t));
        }

        return Piecewise::new(output, Some(self.cuts.clone()));
    }
}

impl Piecewise<Bezier> {
    pub fn fuse_nearby_ends(&self, distance: f64) -> Piecewise<Bezier> {
        let mut iter = self.segs.iter().peekable();
        let mut new_segments = Vec::new();
        while let Some(primitive) = iter.next() {
            for next_primitive in iter.peek() {
                if primitive.end_point().distance(next_primitive.start_point()) <= distance {
                    let mut new_primitive = primitive.to_control_points();
                    new_primitive[3] = next_primitive.start_point();
                    new_segments.push(Bezier::from_points(
                        new_primitive[0],
                        new_primitive[1],
                        new_primitive[2],
                        new_primitive[3],
                    ));
                } else {
                    new_segments.push(primitive.clone());
                }
            }
        }

        return Piecewise::new(new_segments, Some(self.cuts.clone()));
    }

    ///Warning: This currently clobbers cuts.
    pub fn remove_short_segs(&self, len: f64, accuracy: i32) -> Piecewise<Bezier> {
        let mut new_segs = Vec::new();
        for bez in &self.segs {
            let arclen_param = ArcLengthParameterization::from(bez, accuracy);
            if arclen_param.get_total_arclen() > len {
                new_segs.push(bez.clone());
            }
        }

        return Piecewise::new(new_segs, None);
    }

    pub fn split_at_discontinuities(&self, distance: f64) -> Piecewise<Piecewise<Bezier>> {
        let mut output_pws: Vec<Piecewise<Bezier>> = Vec::new();
        let mut current_run: Vec<Bezier> = Vec::new();
        let mut last_bez: Option<Bezier> = None;
        for bez in &self.segs {
            if let Some(lb) = last_bez {
                if lb.end_point().distance(bez.start_point()) < distance {
                    current_run.push(bez.clone());
                } else {
                    let output = Piecewise::new(current_run, None);
                    output_pws.push(output);

                    current_run = Vec::new();
                    current_run.push(bez.clone());
                }
            } else {
                current_run.push(bez.clone());
            }

            last_bez = Some(bez.clone());
        }

        let output = Piecewise::new(current_run, None);
        if !output.segs.is_empty() {
            output_pws.push(output);
        }

        return Piecewise::new(output_pws, None);
    }
}

impl<T: Evaluate<EvalResult = Vector> + Primitive + Send + Sync> Piecewise<T> {
    pub fn is_closed(&self) -> bool {
        if self.start_point().is_near(self.end_point(), SMALL_DISTANCE) {
            return true;
        }
        return false;
    }

    pub fn subdivide(&self, t: f64) -> Piecewise<T> {
        let mut new_segments = Vec::new();
        let mut new_cuts = Vec::new();
        for primitive in &self.segs {
            let subdivisions = primitive.subdivide(t);

            match subdivisions {
                Some(subs) => {
                    new_segments.push(subs.0);
                    new_segments.push(subs.1);
                }
                _ => {
                    new_segments.push(primitive.clone());
                }
            }
        }

        let mut last_cut = None;
        for cut in &self.cuts {
            if let Some(lcut) = last_cut {
                if t > lcut && t < *cut {
                    new_cuts.push(t);
                    last_cut = Some(t);
                } else {
                    last_cut = Some(*cut);
                }
            }

            new_cuts.push(*cut);
        }

        return Piecewise::new(new_segments, Some(new_cuts));
    }

    pub fn cut_at_t(&self, t: f64) -> Piecewise<T> {
        let mut new_segments = Vec::new();
        let mut new_cuts = Vec::new();

        let seg_num = self.seg_n(t);
        let seg_time = self.seg_t(t);

        let iter = self.segs.iter().enumerate();
        for (i, seg) in iter {
            if i == seg_num {
                let subdivisions = seg.subdivide(seg_time);

                match subdivisions {
                    Some(subs) => {
                        new_segments.push(subs.0);
                        new_segments.push(subs.1);
                    }
                    _ => {
                        new_segments.push(self.segs[i].clone());
                    }
                }
            } else {
                new_segments.push(self.segs[i].clone());
            }
        }

        let mut last_cut = None;
        for cut in &self.cuts {
            if let Some(lcut) = last_cut {
                if t > lcut && t < *cut {
                    new_cuts.push(t);
                    last_cut = Some(t);
                } else {
                    last_cut = Some(*cut);
                }
            } else {
                last_cut = Some(*cut);
            }

            new_cuts.push(*cut);
        }

        return Piecewise::new(new_segments, Some(new_cuts));
    }
}

// Returns a primitive and the range of t values that it covers.
pub struct SegmentIterator<T: Evaluate + Primitive + Sized> {
    piecewise: Piecewise<T>,
    counter: usize,
}

impl<T: Evaluate + Primitive + Sized> SegmentIterator<T> {
    pub fn new(pw: Piecewise<T>) -> Self {
        Self {
            piecewise: pw,
            counter: 0,
        }
    }
}

impl<T: Evaluate + Primitive + Sized> Iterator for SegmentIterator<T> {
    type Item = (T, f64, f64); // primitive, start time, end time

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == self.piecewise.segs.len() {
            return None;
        }

        let start_time = self.piecewise.cuts[self.counter];
        let end_time = self.piecewise.cuts[self.counter + 1];
        let item = &self.piecewise.segs[self.counter];

        self.counter = self.counter + 1;
        return Some((item.clone(), start_time, end_time));
    }
}

// Returns a vector (in struct [`Piecewise`]: `segs`) of only continuous Beziers.
// Note: G refers to continuity. G0, G1, G2 continuous Beziers.
// Note: This is an iterator holding type. It starts out empty. The iterator builds it up!
#[derive(Debug, Clone, Default)]
pub struct ContinuousBeziers<const G: u8> {
    /// Input piecewise.
    pub piecewise: Piecewise<Bezier>, // undivided
    // Final piecewise Bezier.
    pub output_pw: Piecewise<Piecewise<Bezier>>,
    // List of grouped Bezier segment indexes on original path.
    pub split_idx_vec: Vec<Vec<usize>>,
}

impl Piecewise<Bezier> {
    // Are input bezier splines A and B connected according to the definition of G^n continuity?
    fn are_beziers_connected<const G: u8>(bezier1: &Bezier, bezier2: &Bezier) -> bool {
        // Simple point comparisons.
        macro_rules! xdist {
            () => {
                (bezier1.w4.x - bezier2.w1.x).abs() < SMALL_DISTANCE
            };
        }
        macro_rules! ydist {
            () => {
                (bezier1.w4.y - bezier2.w1.y).abs() < SMALL_DISTANCE
            };
        }
        // Calculates whether the difference between the tangent at the end point of the first
        // Bezier spline and the tangent at the start point of the second Bezier spline is within
        // four times the Rust f32 epsilon.
        macro_rules! taneq {
            () => {
                ((bezier1.w4 - bezier1.w3) - (bezier2.w1 - bezier2.w2)).abs()
                    < Vector::from(SMALL_DISTANCE)
                    && ((2.0f64 * (bezier1.w3 - 2.0f64 * bezier1.w4 + bezier1.w4))
                        - (2.0f64 * (bezier2.w2 - 2.0f64 * bezier2.w1 + bezier2.w1)).abs()
                        < Vector::from(SMALL_DISTANCE))
            };
        }
        // Calculates whether the difference between the second derivative at the end point of the
        // first Bezier spline and the second derivative at the start point of the second Bezier
        // spline is within four times the Rust f32 epsilon.
        macro_rules! crveq {
            () => {{
                let k1 = (bezier1.w4 - 2.0f64 * bezier1.w3 + bezier1.w2)
                    .cross(&(bezier1.w4 - 3.0f64 * bezier1.w3 + 2.0f64 * bezier1.w2))
                    .normalize()
                    / ((bezier1.w4 - 2.0f64 * bezier1.w3 + bezier1.w2).normalize()
                        * (bezier1.w4 - 3.0f64 * bezier1.w3 + 2.0f64 * bezier1.w2)
                            .normalize()
                            .powf(3.0f64 / 2.0f64));
                let k2 = (bezier2.w1 - 2.0f64 * bezier2.w2 + bezier2.w3)
                    .cross(&(bezier2.w1 - 3.0f64 * bezier2.w2 + 2.0f64 * bezier2.w3))
                    .normalize()
                    / ((bezier2.w1 - 2.0f64 * bezier2.w2 + bezier2.w3).normalize()
                        * (bezier2.w1 - 3.0f64 * bezier2.w2 + 2.0f64 * bezier2.w3)
                            .normalize()
                            .powf(3.0f64 / 2.0f64));
                (k1 - k2).abs() < Vector::from(SMALL_DISTANCE)
            }};
        }
        match G {
            // G^0 continuity
            // The two Bezier splines are connected if the end point of the first Bezier spline
            // equals the start point of the second Bezier spline.
            0 => xdist!() && ydist!(),
            // G^1 continuity
            // The two Bezier splines are connected if the end point of the first Bezier spline
            // equals the start point of the second Bezier spline and the tangent at the end point
            // equals the tangent at the start point.
            1 => xdist!() && ydist!() && taneq!(),
            // G^2 continuity
            // The two Bezier splines are connected if the end point of the first Bezier spline
            // equals the start point of the second Bezier spline and the tangent at the end point
            // equals the tangent at the start point and the second derivative at the end point
            // equals the second derivative at the start point.
            2 => xdist!() && ydist!() && taneq!() && crveq!(),
            const_order => {
                panic!("G continuity of order {} not supported.", const_order);
            }
        }
    }
}

impl Piecewise<Bezier> {
    pub fn are_beziers_connected_g1(bezier1: &Bezier, bezier2: &Bezier) -> bool {
        Self::are_beziers_connected::<1>(bezier1, bezier2)
    }

    pub fn are_beziers_connected_g2(bezier1: &Bezier, bezier2: &Bezier) -> bool {
        Self::are_beziers_connected::<2>(bezier1, bezier2)
    }

    pub fn as_continuous_beziers<const G: u8>(&self) -> ContinuousBeziers<G> {
        ContinuousBeziers {
            piecewise: self.clone(),
            ..Default::default()
        }
    }
}

/// ContinuousBeziersIterator iterates over only continuous Beziers.
/// The output is a vector of piecewise Beziers.
/// Note: G refers to continuity. G0, G1, G2 continuous Beziers.
/// Note: This is an iterator holding type. It starts out empty. The iterator builds it up!
#[derive(Debug, Clone)]
pub struct ContinuousBeziersIterator<const G: u8> {
    inner: ContinuousBeziers<G>,
}

impl<const G: u8> IntoIterator for ContinuousBeziers<G> {
    type Item = (Vec<usize>, Piecewise<Bezier>);
    type IntoIter = ContinuousBeziersIterator<G>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.calculate();
        ContinuousBeziersIterator { inner: self }
    }
}

impl<const G: u8> ContinuousBeziers<G> {
    fn calculate(&mut self) {
        for ((ai, a), (bi, b)) in self.piecewise.segs.iter().enumerate().tuple_windows() {
            if Piecewise::are_beziers_connected::<G>(&a, &b) {
                self.split_idx_vec.last_mut().unwrap().push(bi);
            } else {
                self.split_idx_vec.push(vec![ai, bi]);
            }
        }

        for idx_vec in &self.split_idx_vec {
            let mut pw_bezier_vec = Vec::new();
            // Split up the Piecewise<Bezier>.
            for idx in idx_vec {
                pw_bezier_vec.push(self.piecewise.segs[*idx].clone());
            }
            // Make a new Piecewise<Bezier> from the split up Piecewise<Bezier>.
            let pw_bezier = Piecewise::new(pw_bezier_vec, None);
            self.output_pw.segs.push(pw_bezier);
        }
    }
}

impl<const G: u8> Iterator for ContinuousBeziersIterator<G> {
    type Item = (Vec<usize>, Piecewise<Bezier>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.output_pw.segs.is_empty() {
            return None;
        }

        let item = self.inner.output_pw.segs.remove(0);
        let idx_vec = self.inner.split_idx_vec.remove(0);

        return Some((idx_vec, item));
    }
}
