use glifparser::glif::contour_operations::dash::DashContour;
use glifparser::glif::Glif;
use glifparser::outline::skia::{
    ConicsToCubics as _, FromSkOutline as _, FromSkiaPath as _, SplitSkiaPath as _,
    ToSkiaPath as _, ToSkiaPaths as _,
};
use glifparser::outline::Outline;
use glifparser::point::PointType;
use glifparser::PointData;
use kurbo::Shape as _;
use kurbo::{BezPath as KurboPath, PathEl as KurboEl, Point as KurboPoint};
use skia::{PathEffect, StrokeRec};
use skia_safe as skia;

use log;

fn add_flutter_to_dash_description(desc: &mut Vec<f32>, slen: f32) -> f32 {
    let dash_len = desc.iter().sum::<f32>();
    let gap_len = desc.iter().skip(1).step_by(2).sum::<f32>();
    let s_g = slen / gap_len;
    let s_d = slen / dash_len;
    let m = s_d - s_d.floor();
    let w = if m.is_nan() { 0. } else { dash_len * m };
    let desc_len = desc.len() as f32;
    let w_d = (w / s_d / desc_len) as f32;
    desc.iter_mut().for_each(|f| {
        *f += w_d;
    });
    log::trace!(
        "slen {}, gap_len {} → s/g {} → m {} → w {}, w_d {}",
        slen,
        gap_len,
        s_g,
        m,
        w,
        w_d
    );
    w_d
}

/// slen = path length
fn path_effect_with_flutter(slen: f32, settings: &DashContour) -> PathEffect {
    let mut desc = settings.dash_desc.to_vec();
    let w_d = add_flutter_to_dash_description(&mut desc, slen);
    let flutter = w_d * desc.iter().skip(1).step_by(2).len() as f32;
    if flutter > 1.0 {
        log::warn!(
            "Added a lot of flutter ({}) to dashes ({} over {} segments)",
            flutter,
            w_d,
            desc.len()
        );
    }
    PathEffect::dash(&desc, 0.).unwrap()
}

fn make_dash_effect(skp: &skia::Path, settings: &DashContour) -> Vec<PathEffect> {
    let mut measure = skia::PathMeasure::from_path(&skp, false, None);
    let mut slens = vec![measure.length()];
    while measure.next_contour() {
        slens.push(measure.length());
    }
    let mut ret = vec![];
    for slen in slens {
        ret.push(path_effect_with_flutter(slen, settings));
    }
    ret
}

use std::mem;

pub fn dash_along_path<PD: PointData>(
    outline: &Outline<PD>,
    settings: &DashContour,
) -> Outline<PD> {
    let skp = outline.to_skia_paths(None).combined();
    let path_effects = make_dash_effect(&skp, &settings);

    let mut paint = skia::Paint::default();
    use skia::{PaintCap, PaintJoin};
    paint.set_style(skia::PaintStyle::Stroke);
    paint.set_stroke_width(settings.stroke_width);
    unsafe {
        paint.set_stroke_join(mem::transmute(settings.paint_join));
        paint.set_stroke_cap(mem::transmute(settings.paint_cap as u32));
    }

    let s_r = StrokeRec::from_paint(&paint, skia::PaintStyle::Stroke, 10.);
    let mut s_r_c = s_r.clone();
    if let Some(cull) = settings.cull {
        let mut cull_paint = skia::Paint::default();
        cull_paint.set_stroke_width(cull.width);
        s_r_c = StrokeRec::from_paint(&cull_paint, skia::PaintStyle::Stroke, 10.);
    }
    let mut gpskp: Outline<()> = Outline::from_skia_path(&skp);
    let mut skp_o = skia::Path::new();
    for (i, gpath) in gpskp.iter_mut().enumerate() {
        let mut gpath_o = skia::Path::new();
        let mut s_r = StrokeRec::from_paint(&paint, skia::PaintStyle::Stroke, 10.);
        let path = gpath.to_skia_path(None).unwrap();
        path_effects[i].filter_path_inplace(
            &mut gpath_o,
            &path,
            &mut s_r,
            skia::Rect::new(0., 0., 0., 0.),
        );
        if gpath_o.count_points() == 0 || gpath_o.is_empty() {
            s_r = StrokeRec::from_paint(&paint, skia::PaintStyle::Stroke, 10.);
            s_r.apply_to_path(&mut gpath_o, &path);
        }
        skp_o.add_path(&gpath_o, (0., 0.), None);
    }
    let mut final_skpath = skia::Path::new();

    let gpskp_o: Outline<()> = Outline::from_skia_path(&skp_o);
    for (i, gpath) in gpskp_o.iter().enumerate() {
        let path = gpath.to_skia_path(None).unwrap();
        if i == gpskp_o.len() - 1 && !settings.include_last_path && settings.cull.is_some() {
            break;
        }
        let mut skp_o_s = skia::Path::new();
        if settings.stroke_width != 0.0 {
            s_r.apply_to_path(&mut skp_o_s, &path);
        } else {
            skp_o_s = path.clone();
        }

        if settings.cull.is_some() {
            let mut skp_o_s2 = skia::Path::new();
            s_r_c.set_stroke_params(PaintCap::Square, PaintJoin::Miter, 4.);
            s_r_c.apply_to_path(&mut skp_o_s2, &skp_o_s);
            if let Some(path) = final_skpath.op(&skp_o_s2, skia::PathOp::Difference) {
                final_skpath = path;
            }
        }

        if settings.stroke_width != 0.0 && settings.cull.is_some() {
            match final_skpath.op(&skp_o_s, skia::PathOp::Union) {
                Some(fsk) => {
                    final_skpath = fsk;
                }
                None => {
                    log::error!("Ran into a skia::PathOp::Union that refused to resolve. This is likely a Skia bug; downgrading to overlapping splines. (Consider testing w/MFEKpathops BOOLEAN)");
                    final_skpath.add_path(&skp_o_s, (0., 0.), None);
                }
            }
        } else {
            final_skpath.add_path(&skp_o_s, (0., 0.), None);
        }
    }

    let mut final_output = Outline::new();

    // uses glifparser traits SplitSkiaPath, ConicsToCubics
    let skoutline = final_skpath.split_skia_path().conics_to_cubics();
    for skc in skoutline {
        let mut kurbo_vec = vec![];
        for (pointtype, points, _) in skc.iter() {
            let kurbo_points: Vec<KurboPoint> = points
                .iter()
                .map(|p| KurboPoint::new(p.x as f64, p.y as f64))
                .collect();
            match pointtype {
                PointType::Move => kurbo_vec.push(KurboEl::MoveTo(kurbo_points[0])),
                PointType::Line => kurbo_vec.push(KurboEl::LineTo(kurbo_points[0])),
                PointType::Curve => kurbo_vec.push(KurboEl::CurveTo(
                    kurbo_points[0],
                    kurbo_points[1],
                    kurbo_points[2],
                )),
                _ => (),
            }
        }
        if settings.stroke_width != 0.0 {
            kurbo_vec.push(KurboEl::ClosePath);
        }
        let kpath = KurboPath::from_vec(kurbo_vec);
        let area = if settings.stroke_width != 0.0 {
            kpath.area()
        } else {
            kpath.perimeter(1.)
        };
        if area.abs() > settings.cull.map(|c| c.area_cutoff as f64).unwrap_or(0.) {
            // uses glifparser trait FromSkOutline
            let mut ol = Outline::from_skoutline(vec![skc]);
            if settings.stroke_width == 0.0 {
                ol.first_mut().map(|c| {
                    c.first_mut().map(|p| {
                        p.ptype = PointType::Move;
                    })
                });
            }
            final_output.extend(ol);
        }
    }

    final_output
}

pub fn dash_along_glif<PD: PointData>(glif: &Glif<PD>, settings: &DashContour) -> Glif<PD> {
    let mut glif = glif.clone();

    let final_output = match glif.outline.as_ref().map(|o| dash_along_path(o, settings)) {
        Some(fo) => fo,
        None => return glif.clone(),
    };

    glif.outline = Some(final_output);
    glif
}
