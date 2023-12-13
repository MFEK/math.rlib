#![cfg(feature = "fontforge")]

use fontforge_typeconv::*;
use std::{ffi, fs, ptr};

#[test]
fn convert_glif() {
    eprintln!("Reading nib...");
    let nibglif: glifparser::Glif<()> =
        glifparser::read(&fs::read_to_string("tests/data/nib2.glif").unwrap()).unwrap();
    eprintln!("Reading path...");
    let ssglif: glifparser::Glif<()> =
        glifparser::read(&fs::read_to_string("tests/data/Q_.glif").unwrap()).unwrap();
    let mut outglif = ssglif.clone();
    // The "raw"'s are fontforge::SplineSet's that are having their memory managed by Rust.
    let (nibss_raw, _nibss_ffsps) = glif_to_ffsplineset(nibglif);
    let (ss_raw, _ss_ffsps) = glif_to_ffsplineset(ssglif);
    // These are Rust Box<_> types for holding types that will be transferred to C
    let mut nibss_boxed = Box::new(nibss_raw);
    let mut ss_boxed = Box::new(ss_raw);
    // These are integer null pointers passable to C
    let nibss = nibss_boxed.as_mut();
    let ss = ss_boxed.as_mut();
    let out_ss;
    unsafe {
        let shape = fontforge::NibIsValid(&mut nibss[0] as *mut _);
        if shape != 0 {
            let shapetype = fontforge::NibShapeTypeMsg(shape);
            eprintln!(
                "Shape: {}\nCannot stroke!",
                ffi::CStr::from_ptr(shapetype).to_str().unwrap()
            );
            return;
        }
        let si = fontforge::InitializeStrokeInfo(ptr::null_mut());
        (*si).stroke_type = fontforge::si_type_si_nib;
        (*si).nib = &mut nibss[0] as *mut _;
        (*si).width = 10.;
        (*si).simplify = -1;
        (*si).rmov = fontforge::stroke_rmov_srmov_none;
        // Do the stroke:
        let newss = fontforge::SplineSetStroke(&mut ss[0] as *mut _, si, 0);
        //eprintln!("{:?}", *newss);
        out_ss = *newss;
    }
    let out = ffsplineset_to_outline(out_ss);
    outglif.outline = Some(out);
    //eprintln!("{}", glifparser::write_ufo_glif(outglif));
}
