// These constants taken directly from flo_curves.

/// Length we consider a small distance (points closer than this far apart are considered to be the same)
pub const SMALL_DISTANCE: f64 = 0.001;

/// Length we consider a 'close' distance (we may round to this precision or cut out points that are closer than this)
pub const CLOSE_DISTANCE: f64 = 0.01;

/// Difference between 't' values on a bezier curve for values considered the same
pub const SMALL_T_DISTANCE: f64 = 0.000001;
