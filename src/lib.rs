#![allow(non_snake_case)] // for our name MFEKmath

pub mod vector;
pub mod piecewise;
pub mod rect;
pub mod bezier;
pub mod arclenparameterization;
pub mod consts;
pub mod evaluate;
pub mod parameterization;
pub mod polar;
pub mod primitive;
pub mod coordinate;
pub mod interpolator;
pub mod glyphbuilder;
#[cfg(feature="default")]
pub mod variable_width_stroking;
#[cfg(feature="default")]
pub mod pattern_along_path;
#[cfg(feature="default")]
pub mod dash_along_path;
#[cfg(feature="default")]
pub use {skia_safe, self::{variable_width_stroking::*, pattern_along_path::*, dash_along_path::*}};

pub use self::vector::Vector;
pub use self::piecewise::Piecewise;
pub use self::rect::Rect;
pub use self::bezier::Bezier;
pub use self::parameterization::Parameterization;
pub use self::arclenparameterization::ArcLengthParameterization;
pub use self::glyphbuilder::GlyphBuilder;
pub use self::primitive::Primitive;

pub use self::evaluate::Evaluate;
pub use self::evaluate::{EvalScale, EvalRotate, EvalTranslate};
pub use self::interpolator::{Interpolator, InterpolationType};
