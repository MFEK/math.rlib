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

// This struct models a simple piecewise function. It maps 0-1 such that 0 is the beginning of the first curve
// in the collection and 1 is the end of the last.
#[derive(Clone, Debug)]

pub struct Piecewise<T: Evaluate> {
    // This supports 
    pub cuts: Vec<f64>,
    // this should definitely change to private at some point with an iterator or getter to access
    pub segs: Vec<T>
}

impl<T: Evaluate> Piecewise<T> {
    pub fn new(segs: Vec<T>, cuts: Option<Vec<f64>>) -> Self
    {
        match cuts {
            Some(cuts) => {
                return Self {
                    cuts,
                    segs
                }
            }
            
            // if we are given just a list of segments we generate the cuts ourselves
            _ => {
                let mut out_cuts: Vec<f64> = Vec::new();

                out_cuts.push(0.);

                let seg_iter = segs.iter().enumerate().peekable();
                let seg_len = segs.len();

                for (i, _seg) in seg_iter {
                    out_cuts.push((i+1) as f64 / seg_len as f64);
                }

                return Self {
                    cuts: out_cuts,
                    segs
                }
            }
        }
    }

    // implementation ripped from lib2geom, performs a binary search to find our segment
    pub fn seg_n(&self, t: f64) -> usize
    {
        let mut left = 0;
        let mut right = self.cuts.len() - 1;

        while left < right {
            let middle = (right+left)/2;

            if left == middle { return middle; }
            if right == middle { return left; }
            if self.cuts[middle] == t { return middle };

            if self.cuts[middle] < t {
                left = middle
            } else {
                right = middle;
            }
        }

        // This needs to be replaced with success/failure.
        panic!("Couldn't find the target segment!");
    }

    pub fn seg_t(&self, t: f64) -> f64 
    {
        let i = self.seg_n(t);
        return (t - self.cuts[i]) / (self.cuts[i+1] - self.cuts[i]);
    }
}

// TODO: Move these functions to a more appropriate submodule.
impl<T: Evaluate<EvalResult = Vector>+Primitive+Send+ Sync> Piecewise<Piecewise<T>>
{
    // we split the primitive that contains t at t
    pub fn subdivide(&self, t: f64) -> Self
    {
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
                    new_segments.push(Bezier::from_points(new_primitive[0], new_primitive[1], new_primitive[2], new_primitive[3]));
                }
                else
                {
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
            }
            else {
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

impl<T: Evaluate<EvalResult = Vector>+Primitive+Send+Sync> Piecewise<T>
{    

    pub fn is_closed(&self) -> bool
    {
        if self.start_point().is_near(self.end_point(),SMALL_DISTANCE)
        {
            return true;
        }
        return false;
    }
    
    pub fn subdivide(&self, t: f64) -> Piecewise<T>
    {
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

    pub fn cut_at_t(&self, t: f64) -> Piecewise<T>
    {
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
pub struct SegmentIterator<T: Evaluate+Primitive+Sized> {
    piecewise: Piecewise<T>,
    counter: usize
}

impl<T: Evaluate+Primitive+Sized> SegmentIterator<T> {
    pub fn new(pw: Piecewise<T>) -> Self {
        Self {
            piecewise: pw,
            counter: 0
        }
    }
}

impl<T: Evaluate+Primitive+Sized> Iterator for SegmentIterator<T> {
    type Item = (T, f64, f64); // primitive, start time, end time

    fn next(&mut self) -> Option<Self::Item>
    {
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
