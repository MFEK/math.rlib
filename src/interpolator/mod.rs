mod evaluate;

pub struct Interpolator {
    start: f64,
    finish: f64,
    func: fn(&Self, f64) -> f64,
}

#[derive(Clone, Copy)]
pub enum InterpolationType {
    Null,
    Linear,
}

impl Interpolator {
    fn interpolate_none(&self, _t: f64) -> f64 {
        self.start
    }

    fn interpolate_linear(&self, t: f64) -> f64 {
        (1. - t) * self.start + t * self.finish
    }

    fn interpolate_exponential(&self, t: f64) -> f64 {
        self.start + (self.finish - self.start) * t * t
    }

    pub fn new(start: f64, finish: f64, kind: InterpolationType) -> Self {
        match kind {
            InterpolationType::Null => Self::new_none(start, finish),
            InterpolationType::Linear => Self::new_linear(start, finish),
        }
    }

    pub fn new_none(start: f64, finish: f64) -> Self {
        Interpolator {
            start,
            finish,
            func: Self::interpolate_none,
        }
    }

    pub fn new_linear(start: f64, finish: f64) -> Self {
        Interpolator {
            start,
            finish,
            func: Self::interpolate_linear,
        }
    }

    pub fn new_exponential(start: f64, finish: f64) -> Self {
        Interpolator {
            start,
            finish,
            func: Self::interpolate_exponential,
        }
    }
}
