pub mod vector;
pub mod piecewise;
pub mod rect;
pub mod bezier;
pub mod arclenparameterization;
pub mod consts;
pub mod evaluate;
pub mod parameterization;
pub mod coordinate;
pub mod interpolator;
pub mod glyphbuilder;
pub mod variable_width_stroking;
pub mod pattern_along_path;

extern crate skia_safe;

pub use self::vector::Vector;
pub use self::piecewise::Piecewise;
pub use self::rect::Rect;
pub use self::bezier::Bezier;
pub use self::arclenparameterization::ArcLengthParameterization;
pub use self::glyphbuilder::GlyphBuilder;

pub use self::variable_width_stroking::*;

pub use self::evaluate::Evaluate;
pub use self::evaluate::{EvalScale, EvalRotate, EvalTranslate};

pub use self::parameterization::Parameterization;

pub use self::interpolator::{Interpolator, InterpolationType};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
