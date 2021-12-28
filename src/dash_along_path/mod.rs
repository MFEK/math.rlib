use glifparser::{self, Glif, PointType};
use glifparser::glif::DashContour;
use glifparser::outline::skia::{FromSkiaPath as _, ToSkiaPath as _, ToSkiaPaths as _, SplitSkiaPath as _, ConicsToCubics as _, FromSkOutline as _};
use kurbo::{Point as KurboPoint, BezPath as KurboPath, PathEl as KurboEl};
use kurbo::Shape as _;
use skia_safe as skia;
use skia::{PathEffect, StrokeRec};

use log;

fn make_dash_effect(skp: &skia::Path, dash_desc: &[f32]) -> PathEffect {
    let mut measure = skia::PathMeasure::from_path(&skp, false, None);
    let mut slen = measure.length();
    while measure.next_contour() {
        slen += measure.length();
    }
    let mut desc = dash_desc.to_vec();
    let dash_len: f32 = desc.iter().sum();
    let s_g = slen / dash_len;
    let m = s_g - s_g.floor();
    let w = if m.is_nan() { dash_len } else { dash_len * m };
    let w_d = w / s_g as f32;
    desc.iter_mut().skip(1).step_by(2).for_each(|f|{ *f+=w_d });
    log::trace!("slen {}, dash_len {} → s/g {} → m {} → w {}, w_d {}", slen, dash_len, s_g, m, w, w_d);
    let w_a = w_d * desc.len() as f32;
    if w_a > 1.0 {
        log::warn!("Added {} of flutter to dashes ({} over {} segments)", w_a, w_d, desc.len());
    }
    PathEffect::dash(&desc, 0.).unwrap()
}

use std::mem;

pub fn dash_along_glif<PD: glifparser::PointData>(glif: &Glif<PD>, settings: &DashContour) -> Glif<PD> {
    let oglif = glif; // keep a reference to original data around
    let mut glif = glif.clone();
    let skp = glif.outline.unwrap().to_skia_paths(None).combined();
    let p_e = make_dash_effect(&skp, &settings.dash_desc);

    let mut paint = skia::Paint::default();
    use skia::{PaintJoin, PaintCap};
    paint.set_style(skia::PaintStyle::Stroke);
    paint.set_stroke_width(settings.stroke_width);
    unsafe {
        paint.set_stroke_join(mem::transmute(settings.paint_join));
        paint.set_stroke_cap(mem::transmute(settings.paint_cap as u32));
    }

    let mut s_r = StrokeRec::from_paint(&paint, skia::PaintStyle::Stroke, 10.);
    let mut s_r_c = s_r.clone();
    if let Some(cull) = settings.cull {
        paint.set_stroke_width(cull.width);
        s_r_c = StrokeRec::from_paint(&paint, skia::PaintStyle::Stroke, 10.);
    }
    let mut skp_o = skia::Path::new();
    p_e.filter_path_inplace(&mut skp_o, &skp, &mut s_r, skia::Rect::new(0., 0., 0., 0.));
    let mut final_skpath = skia::Path::new();

    if settings.cull.is_some() {
        let gpskp_o: glifparser::Outline::<()> = glifparser::Outline::from_skia_path(&skp_o);

        for (i, gpath) in gpskp_o.iter().enumerate() {
            if i == gpskp_o.len() - 1 && !settings.include_last_path {
                break;
            }
            let path = gpath.to_skia_path(None).unwrap();
            let mut skp_o_s = skia::Path::new();
            if settings.stroke_width != 0.0 {
                s_r.apply_to_path(&mut skp_o_s, &path);
            } else {
                skp_o_s = path.clone();
            }
            let mut skp_o_s2 = skia::Path::new();
            s_r_c.set_stroke_params(PaintCap::Square, PaintJoin::Miter, 4.);
            s_r_c.apply_to_path(&mut skp_o_s2, &skp_o_s);
            if let Some(path) = final_skpath.op(&skp_o_s2, skia::PathOp::Difference) {
                final_skpath = path;
            }

            if settings.stroke_width != 0.0 {
                match final_skpath.op(&skp_o_s, skia::PathOp::Union) {
                    Some(fsk) => { final_skpath = fsk; }
                    None => {
                        log::error!("While working on glif `{}`, ran into a skia::PathOp::Union that refused to resolve. This is likely a Skia bug; downgrading to overlapping splines. (Consider testing w/MFEKpathops BOOLEAN)", &oglif.name);
                        final_skpath.add_path(&skp_o_s, (0., 0.), None);
                    }
                }
            } else {
                final_skpath.add_path(&skp_o_s, (0., 0.), None);
            }
        }
    } else {
        if settings.stroke_width != 0.0 {
            s_r.apply_to_path(&mut final_skpath, &skp_o);
        } else {
            final_skpath = skp_o;
        }
    }

    let mut final_output = glifparser::Outline::new();

    // uses glifparser traits SplitSkiaPath, ConicsToCubics
    let skoutline = final_skpath.split_skia_path().conics_to_cubics();
    for skc in skoutline {
        let mut kurbo_vec = vec![];
        for (pointtype, points, _) in skc.iter() {
            let kurbo_points: Vec<KurboPoint> = points.iter().map(|p|KurboPoint::new(p.x as f64, p.y as f64)).collect();
            match pointtype {
                PointType::Move => kurbo_vec.push(KurboEl::MoveTo(kurbo_points[0])),
                PointType::Line => kurbo_vec.push(KurboEl::LineTo(kurbo_points[0])),
                PointType::Curve => kurbo_vec.push(KurboEl::CurveTo(kurbo_points[0], kurbo_points[1], kurbo_points[2])),
                _ => ()
            }
        }
        if settings.stroke_width != 0.0 {
            kurbo_vec.push(KurboEl::ClosePath);
        }
        let kpath = KurboPath::from_vec(kurbo_vec);
        let area = if settings.stroke_width != 0.0 { kpath.area() } else { kpath.perimeter(1.) };
        if area.abs() > settings.cull.map(|c|c.area_cutoff as f64).unwrap_or(0.) {
            // uses glifparser trait FromSkOutline
            let mut ol = glifparser::Outline::from_skoutline(vec![skc]);
            if settings.stroke_width == 0.0 {
                ol.first_mut().map(|c|c.first_mut().map(|p|{p.ptype = PointType::Move;}));
            }
            final_output.extend(ol);
        }
    }

    glif.outline = Some(final_output);
    glif
}
