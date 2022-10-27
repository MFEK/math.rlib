use fontforge_typeconv::*;
use glifparser;
use std::ffi;
use std::fs;
use std::path::PathBuf;
use std::ptr;

#[derive(Clone, Debug)]
pub struct NibSettings {
    pub nib: PathBuf,
    pub path: PathBuf,
    pub accuracy: f64,
    pub quiet: bool,
}

pub fn convert_glif(settings: &NibSettings) -> Option<String> {
    if !settings.quiet {
        eprintln!("Reading nib...");
    }
    let nibglif: glifparser::Glif<()> = glifparser::read(&fs::read_to_string(&settings.nib).expect("Nib .glif inaccessible")).unwrap();
    if !settings.quiet {
        eprintln!("Reading path...");
    }
    let ssglif: glifparser::Glif<()> =
        glifparser::read(&fs::read_to_string(&settings.path).expect("Path to stroke .glif inaccessible")).unwrap();

    if ssglif.outline.is_none() {
        return Some(glifparser::write(&ssglif).unwrap());
    }

    let mut outglif = ssglif.clone();
    // The "raw"'s are fontforge::SplineSet's that are having their memory managed by Rust.
    let (nibss_raw, _nibss_ffsps) = glif_to_ffsplineset(nibglif);
    let (mut ss_raw, _ss_ffsps) = glif_to_ffsplineset(ssglif);
    // These are Rust Box<_> types for holding types that will be transferred to C
    let mut nibss_boxed = Box::new(nibss_raw[0]);
    let mut ss_vec: Vec<Box<_>> = ss_raw.iter_mut().map(|v| Box::new(v)).collect();
    // These are integer null pointers passable to C
    let nibss = nibss_boxed.as_mut();
    let mut out_ss = vec![];
    unsafe {
        let shape = fontforge::NibIsValid(nibss);
        if shape != 0 {
            let shapetype = fontforge::NibShapeTypeMsg(shape);
            eprintln!("Shape: {}\nCannot stroke!", ffi::CStr::from_ptr(shapetype).to_str().unwrap());
            return None;
        }
        let si = fontforge::InitializeStrokeInfo(ptr::null_mut());
        (*si).stroke_type = fontforge::si_type_si_nib;
        (*si).nib = nibss;
        (*si).width = 10.;
        (*si).accuracy_target = settings.accuracy;
        (*si).simplify = -1;
        (*si).rmov = fontforge::stroke_rmov_srmov_none;
        // Do the stroke for each contour. We do it this way to avoid constructing linked lists of
        // SplineSet's. It seems more reliable:
        for ss in ss_vec.iter_mut() {
            let newss = fontforge::SplineSetStroke(*(ss.as_mut()), si, 0);
            if newss == ptr::null_mut() {
                eprintln!("SplineSetStroke returned NULL. Try to recreate the bug in FontForge. If it happens there, report it upstream to FontForge. Otherwise, report it to MFEKstroke bug tracker.");
            }
            //eprintln!("{:?}", *newss);
            out_ss.push(*newss);
        }
    }
    let mut outlines = vec![];
    for oss in out_ss.iter() {
        outlines.push(ffsplineset_to_outline(*oss));
    }
    let mut outline = vec![];
    for o in outlines {
        outline.extend(o);
    }
    outglif.outline = Some(outline);
    Some(glifparser::write(&outglif).unwrap())
}
