#![allow(non_snake_case)] // for our name MFEKmath
pub mod arclenparameterization;
pub mod bezier;
pub mod consts;
pub mod coordinate;
#[cfg(feature = "skia")]
pub mod dash_along_path;
pub mod evaluate;
pub mod fit_to_points;
#[cfg(feature = "fontforge")]
pub mod nib_stroking;
pub(crate) mod fixup;
pub mod glyphbuilder;
pub mod parameterization;
#[cfg(feature = "skia")]
pub mod pattern_along_path;
pub mod piecewise;
pub mod polar;
pub mod rect;
pub mod variable_width_stroking;
pub mod vector;
pub mod mfek;
pub mod angleparameterization;
pub mod subdivide;
pub mod quadbezier;

#[cfg(feature = "skia")]
pub use {
    self::{dash_along_path::*, pattern_along_path::*, variable_width_stroking::*},
    skia_safe,
};

pub use self::arclenparameterization::ArcLengthParameterization;
pub use self::angleparameterization::AngleParameterization;
pub use self::bezier::Bezier;
pub use self::glyphbuilder::GlyphBuilder;
pub use self::parameterization::Parameterization;
pub use self::piecewise::Piecewise;
pub use self::rect::Rect;
pub use self::vector::Vector;

pub use self::evaluate::Evaluate;
pub use self::evaluate::{EvalRotate, EvalScale, EvalTranslate};
pub use self::fixup::Fixup;
