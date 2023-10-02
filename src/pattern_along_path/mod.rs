use std::vec;

use super::{ArcLengthParameterization, Bezier, Evaluate, EvalScale, EvalTranslate, Parameterization, Piecewise, Vector, Rect};
use super::{AngleParameterization};
use crate::{vec2, angleparameterization, Primitive};

use flo_curves::bezier::curve_intersects_ray;
use glifparser::outline::IntoKurbo;
use glifparser::outline::skia::{ToSkiaPaths, FromSkiaPath};
use glifparser::{Glif, Outline, MFEKPointData};
use glifparser::glif::{Lib};
use glifparser::glif::contour_operations::pap::{PAPContour, PatternCopies, PatternSubdivide, PatternCulling, PatternStretch};
use kurbo::Shape;
use skia_safe::{Path, Paint, Color4f, PaintStyle, StrokeRec, PaintCap, PaintJoin, PathMeasure};

// At some point soon I want to restructure this algorithm. The current two pass 
#[derive(Debug, Clone)]
pub struct PatternSettings {
    pub copies: PatternCopies,
    pub subdivide: PatternSubdivide,
    pub is_vertical: bool, // TODO: Implement this. Might replace it with a general rotation parameter to make it more useful.
    pub stretch: PatternStretch,
    pub spacing: f64,
    pub simplify: bool,
    pub normal_offset: f64,
    pub tangent_offset: f64,
    pub pattern_scale: Vector,
    pub center_pattern: bool,
    pub cull_overlap: PatternCulling,
    pub two_pass_culling: bool,
    pub reverse_culling: bool,
    pub reverse_path: bool,
    pub warp_pattern: bool,
    pub split_path: bool, 
}

struct Span(f64, f64);

fn prepare_pattern(pattern: &Piecewise<Piecewise<Bezier>>, settings: &PatternSettings) -> Piecewise<Piecewise<Bezier>> {
    let mut working_pattern = pattern.clone();
    let pattern_bounds = working_pattern.bounds();
    let pattern_height = f64::abs(pattern_bounds.bottom - pattern_bounds.top);

    if settings.center_pattern {
        let pattern_offset_x = -pattern_bounds.left as f64 - 1.;
        let pattern_offset_y = -pattern_bounds.bottom as f64 - 1.;

        working_pattern = working_pattern.translate(vec2!(pattern_offset_x, pattern_offset_y - pattern_height/2.));
    }

    working_pattern = working_pattern.scale(vec2!(settings.pattern_scale.x, settings.pattern_scale.y));

    // if we've got a simple split we just do that now 
    match settings.subdivide {
        PatternSubdivide::Simple(times) => {
            for _n in 0..times {
                working_pattern = working_pattern.subdivide(0.5);
            }
        }
        _ => {} // We're gonna handle the other options later in the process.
    }

    working_pattern
}

// This function returns a list of spans that represent the locations of the pattern along the path. If the warp setting is on
// we'll lay out patterns along these spans and then warp them to the path. If it's off we'll just translate the pattern to the center
// point of each span.
fn layout_spans(
    pattern: &Piecewise<Piecewise<Bezier>>,
    arclenparam: &ArcLengthParameterization,
    settings: &PatternSettings,
    start_padding: f64,
    end_padding: f64
) 
    -> Vec<Span>
{
    let mut output: Vec<Span> = Vec::new();
    
    // Your other code remains mostly the same...
    let pattern_bounds = pattern.bounds();
    let pattern_width = f64::abs(pattern_bounds.left - pattern_bounds.right);

    let total_arclen = arclenparam.get_total_arclen() - (start_padding + end_padding);
    let total_width = pattern_width + settings.spacing;

    match settings.copies {
        PatternCopies::Single => {
            let mut single_width = total_width;
            if settings.stretch == PatternStretch::On { single_width = total_arclen }
            if f64::floor((total_arclen - start_padding - end_padding) / single_width) > 0. {
                output.push(Span(start_padding, start_padding + single_width));
            }
        },

        PatternCopies::Repeated => {
            let mut copies = (total_arclen / pattern_width) as i32;
            let mut left_over = total_arclen / pattern_width - copies as f64;
            let mut additional_spacing = 0.;

            while left_over < (settings.spacing / pattern_width) * (copies - 1) as f64 {
                copies -= 1;
                left_over = total_arclen / pattern_width - copies as f64;
            }
            left_over = left_over - (settings.spacing / pattern_width) * (copies - 1) as f64;

            let mut stretch_len = 0.;

            match settings.stretch {
                PatternStretch::On => {
                    stretch_len = left_over / copies as f64;
                },
                PatternStretch::Spacing => {
                    additional_spacing = left_over / copies as f64 * pattern_width;
                },
                PatternStretch::Off => {}
            }

            for n in 0..copies {
                let start = start_padding + n as f64 * total_width + n as f64 * stretch_len * pattern_width + n as f64 * additional_spacing;
                let end = start + pattern_width + stretch_len * pattern_width;
                output.push(Span(start, end));
            }
        },
    }

    output
}



// https://www.khanacademy.org/math/multivariable-calculus/integrating-multivariable-functions/line-integrals-in-vector-fields-articles/
// This Khan Academy module has a few classes that really help with understanding the math here. Check out the classes on arc-length and
// the class on getting normals from curve surfaces for more background.

// https://stackoverflow.com/questions/25453159/getting-consistent-normals-from-a-3d-cubic-bezier-path
// This stackoverflow answer is very hepful to understand how normal generation works for bezier curves. It's in 3d but the math is the same.

// http://www.planetclegg.com/projects/WarpingTextToSplines.html
// This blog post was my reference for the overall algorithm. The math structures are inspired loosely by libgeom (used in inkscape). 
// The inkscape implemnetation seems to be very similar to the algorithm described above. The aim is that this implementation gives
// comparable outputs to inkscape's.
#[allow(non_snake_case)]
fn pattern_along_path(path: &Piecewise<Bezier>, pattern: &Piecewise<Piecewise<Bezier>>, settings: &PatternSettings, cull_cache: &mut skia_safe::Path, padding: f64) -> Piecewise<Piecewise<Bezier>>
{
    let balanced_path = path.balance();
    let path: &Piecewise<Bezier> = &balanced_path;

    // let's calculate the area of the pattern for later use
    let mut pattern_area = 0.;
    for contour in pattern.segs.iter() {
        pattern_area += contour.approximate_area().abs();
    }
    let pattern_area = pattern_area;

    if pattern.segs.len() == 0 || settings.pattern_scale.x == 0. || settings.pattern_scale.y == 0. {
        return Piecewise::new(vec![], None);
    }

    // we're gonna parameterize the input path such that 0-1 = 0 -> totalArcLength
    // this is important because samples will be spaced equidistant along the input path
    let arclenparam = ArcLengthParameterization::from(path, 1000);
    let angleparameterization = match settings.subdivide {
        PatternSubdivide::Angle(_) => {
            Some(AngleParameterization::from(path, 1000))
        },
        _ => {
            None
        }
    };
    let angle_intervals = match settings.subdivide {
        PatternSubdivide::Angle(angle) => {
            if angle > 0. {
                Some(angleparameterization.as_ref().unwrap().find_parameters_for_angle_intervals(f64::to_radians(angle)))
            } else {
                None
            }
        },
        _ => {
            None
        }
    };

    let total_arclen = arclenparam.get_total_arclen();

    let working_pattern: Piecewise<Piecewise<Bezier>> = prepare_pattern(pattern, settings);
    let mut spans = layout_spans(&working_pattern, &arclenparam, settings, padding, 0.);

    let pattern_bounds = working_pattern.bounds();
    let pattern_width = f64::abs(pattern_bounds.left - pattern_bounds.right);

    if settings.reverse_culling { spans.reverse() };

    let mut output_segments = Outline::new();

    for span in spans {
        // This is the transform that we'll use to warp the pattern to the path. 
        let warp_to_span = |point: &Vector| {
            // Calculate where along the path we are, if we're warping the path we'll use the x value of the point relative to the pattern width
            // if we're not warping the path we'll use the center of the span across the entire input pattern so that it is not distorted.
            let u = span.0 + (span.1 - span.0) * (point.x / pattern_width);

            // If the path needs to be reversed, modify u accordingly
            let u = if settings.reverse_path { 1. - u } else { u };

            // Parameterize u such that 0-1 maps to the curve by arclength
            let t = arclenparam.parameterize(u / total_arclen);
            let path_point = path.at(t);

            // the derivative here is essentially a velocity or tangent line on the point we're evaulating
            // it faces in the direction of travel along the path
            let d = path.tangent_at(t);
    
            // we rotate the vector by 90 degrees so that it's perpendicular to the direction of travel along the curve
            // normalize the vector and now we've got a unit vector perpendicular to the curve's surface in 'curve space'
            let N = Vector{x: d.y, y: -d.x}.normalize();
    
            // now we multiply this by the y value of the pattern this gives us a point
            // that is as far away from the curve as the input is tall in the direction of the
            // surface normal of the curve.
            let mut P = N * point.y;
    
            // Offset the point by the tangent offset setting.
            P = P + d.normalize() * settings.tangent_offset;
    
            // We offset the point by the normal offset setting.
            P = P + N * settings.normal_offset;
    
            // Now we add the evaluation of the bezier's point to the offset point 
            // this essentially translates P from 'curve space' where 0,0 is the point on the curve
            // at t being evaluated to 'world space' where 0,0 is relative to the glyph origin
            P + path_point
        };

        // When warp is off we just translate the pattern to the center of the span, and rotate it to match the path's tangent
        let stamp_to_span = |point: &Vector| {
            // Calculate the midpoint of the span
            let u_mid = span.0 + (span.1 - span.0) * 0.5;
            
            // Parameterize u_mid such that 0-1 maps to the curve by arclength
            let t_mid = arclenparam.parameterize(u_mid / total_arclen);
            let path_point_mid = path.at(t_mid);
        
            // Derivative (tangent) at the midpoint
            let d_mid = path.tangent_at(t_mid);
            
            // Generate the normal (perpendicular vector) based on the derivative
            let N_mid = Vector { x: d_mid.y, y: -d_mid.x }.normalize();
        
            // Translate the point in the direction of the normal
            let mut P = N_mid * point.y;
            P = P + d_mid.normalize() * (point.x - pattern_width/2.);
            
            // Offset the point by the tangent offset setting
            P = P + d_mid.normalize() * settings.tangent_offset;
            
            // We offset the point by the normal offset setting
            P = P + N_mid * settings.normal_offset;
        
            // Add the midpoint of the bezier's point to the offset point
            // this essentially translates P to 'world space' where 0,0 is relative to the glyph origin
            P + path_point_mid
        };

        let mut working_pattern = working_pattern.clone();

        // if our subdivide mode is angle we need to subdivide the pattern at intervals where the absolute change in
        // angle is greater than the angle parameter, conservatively subdividing the pattern
        if let Some(_) = &angleparameterization {
            match settings.subdivide {
                PatternSubdivide::Angle(_) => {
                    let angle_intervals = angle_intervals.as_ref().unwrap();

                    // first we need to convert the span's time parameters from 0->arclength to 0->1
                    let span = (span.0 / total_arclen, span.1 / total_arclen);

                    // now we need to take those times and find the unparameterized time parameter
                    let span = (arclenparam.parameterize(span.0), arclenparam.parameterize(span.1));

                    // min and max t1, t2 to 0->1 range because floating point math is hard
                    let span = (span.0.min(1.).max(0.), span.1.min(1.).max(0.));

                    let mut new_segments: Vec<Piecewise<Bezier>> = Vec::new();

                    // then for each interval we need to subdivide the beziers where they cross the interval
                    // we do this by employing flo_curves to check for intersection against a vertical line 
                    for contour in working_pattern.segs.iter_mut() {
                        let mut new_contour: Vec<Bezier> = Vec::new();
                        for bez in contour.segs.iter_mut() {
                            let mut intersections = Vec::new();
                            for interval in angle_intervals.iter() {
                                if interval > &span.0 && interval < &span.1 {
                                    let x = (interval - span.0) / (span.1 - span.0) * pattern_width;
                                    let ray_intersections = curve_intersects_ray(bez, &(vec2![x, 0.], vec2!(x, 1.)));

                                    for intersection in ray_intersections {
                                        intersections.push(intersection.0);
                                    }
                                }
                            }

                            // next we wanna split this bezier at each of the curve_times
                            let results = bez.split_at_multiple_t(intersections);
                            // and then we'll add the results to our new contour
                            for result in results {
                                new_contour.push(result);
                            }
                        }
                        new_segments.push(Piecewise::new(new_contour, None));
                    }

                    working_pattern = Piecewise::new(new_segments, None);
                },
                _ => {}
            }
        }

        let transformed_pattern = if settings.warp_pattern {
            working_pattern.apply_transform(warp_to_span)
        } else {
            working_pattern.apply_transform(stamp_to_span)
        };

        match settings.cull_overlap {
            PatternCulling::Off => {
                for contour in transformed_pattern.segs {
                    output_segments.push(contour.to_contour::<MFEKPointData>());
                }
            },
            PatternCulling::RemoveOverlapping => {
                // Okay we've applied our transform, and have our pattern so what's next is we convert it to a skia path
                // and then we'll use skia pathops to check for overlaps and cull them.
                let skpattern: Path = transformed_pattern.clone().to_skpath();
                let intersection = skpattern.op(&cull_cache, skia_safe::PathOp::Intersect);

                let mut found_overlap = false;
                if let Some(intersection) = intersection {
                    if !intersection.is_empty() {
                        found_overlap = true;
                    }
                }

                if !found_overlap {
                    // we found no overlap so we can just add the pattern to the output
                    for contour in transformed_pattern.segs {
                        output_segments.push(contour.to_contour());
                    }

                    // and add the pattern to the cull cache
                    cull_cache.reverse_add_path(&skpattern);
                }
            },
            PatternCulling::EraseOverlapping(stroke_width, cull_area_percent) => {
                let outline_pattern = transformed_pattern.to_outline::<MFEKPointData>();
                let skia_paths_pattern = outline_pattern.to_skia_paths(None);

                if let Some(closed) = skia_paths_pattern.closed {
                    // We convert our pattern to a skia path
                    let skpattern = closed;

                    let mut local_output_segments = vec![];
                    // Then we get the difference between the cull cache and the pattern
                    let difference = skpattern.op(&cull_cache, skia_safe::PathOp::Difference);
                    if let Some(difference) = difference {
                        let culled_pattern: Vec<Vec<glifparser::Point<MFEKPointData>>> = Outline::from_skia_path(&difference);
                        let culled_pattern_pw = Piecewise::from(&culled_pattern);

                        for contour in culled_pattern_pw.segs {
                            let area = contour.approximate_area().abs();

                            if area > cull_area_percent / 100. * pattern_area {
                                local_output_segments.push(contour.to_contour::<MFEKPointData>());
                            }
                        }
                    }
                    
                    for contour in local_output_segments {
                        output_segments.push(contour);
                    }

                    let skpattern = local_output_segments.to_skia_paths(None);
                    // After culling the pattern we need to add it to the cull cache
                    // Create a Paint object to configure the stroke
                    let mut paint = Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
                    paint.set_style(PaintStyle::Stroke);
                    paint.set_stroke_width(stroke_width as f32);  // Set stroke width to 10

                    // Initialize a StrokeRec object with the Paint settings
                    let mut stroke_rec = StrokeRec::from_paint(&paint, PaintStyle::Stroke, 10.0);
                    stroke_rec.set_stroke_params(PaintCap::Square, PaintJoin::Round, 4.);
                                                        
                    // Create a new path to hold the stroked path
                    let mut stroked_path = Path::new();

                    // Get the stroke path
                    stroke_rec.apply_to_path(&mut stroked_path, &skpattern);

                    let unioned_stroke_pattern = stroked_path.op(&skpattern, skia_safe::PathOp::Union).unwrap();
                    // add it to the cull cache
                    //cull_cache.add_path(&skpattern, (0., 0.), skia_safe::path::AddPathMode::Append);
                    cull_cache.add_path(&unioned_stroke_pattern, (0., 0.), skia_safe::path::AddPathMode::Append);
                    
                }
            },
        }

    }

    let mut result_pw = Piecewise::from(&output_segments);
    if settings.simplify {
        let skpattern: Path = result_pw.to_skpath();
        result_pw = Piecewise::from(&skpattern.simplify().unwrap().as_winding().unwrap());
    }

    return result_pw;
}

// Called by both pap_mfek and pap_ufo this splits the input paths at discontinuities according to the settings and
// then calls pattern_along_path on each segment.
pub fn split_and_blit(path: &Piecewise<Bezier>, pattern: &Piecewise<Piecewise<Bezier>>, settings: &PatternSettings, cull_cache: &mut skia_safe::Path) -> Piecewise<Piecewise<Bezier>> {
    let split_path = if settings.split_path {
        path.split_at_tangent_discontinuities(0.01)
    } else {
        Piecewise::new(vec![path.clone()], None)
    };

    let mut output_segments: Vec<Piecewise<Bezier>> = Vec::new();
    
    let mut first = true;
    for (i, segment) in split_path.segs.iter().enumerate() {
        let padding = if first && !path.is_closed() { 0. } else { settings.spacing };
        first = false;

        let result_pw = pattern_along_path(&segment, pattern, &settings, cull_cache, padding);

        for result_seg in result_pw.segs
        {
            output_segments.push(result_seg.clone());
        }
    }

    return Piecewise::new(output_segments, None);
}

pub fn pattern_along_path_mfek<PD: glifparser::PointData>(path: &Piecewise<Bezier>, settings: &PAPContour<PD>) -> Piecewise<Piecewise<Bezier>>
{
    // we're only doing this to avoid a circular dependency
    let split_settings = PatternSettings {
        copies: settings.copies.clone(),
        subdivide: settings.subdivide.clone(),
        is_vertical: settings.is_vertical,
        stretch: settings.stretch,
        spacing: settings.spacing,
        simplify: settings.simplify,
        normal_offset: settings.normal_offset,
        tangent_offset: settings.tangent_offset,
        pattern_scale: Vector{ x: settings.pattern_scale.0, y: settings.pattern_scale.1},
        center_pattern: settings.center_pattern,
        cull_overlap: settings.prevent_overdraw,
        two_pass_culling: settings.two_pass_culling,
        reverse_path: settings.reverse_path,
        reverse_culling: settings.reverse_culling,
        warp_pattern: settings.warp_pattern,
        split_path: settings.split_path,
    };

    let mut cull_dummy = skia_safe::Path::new();
    return split_and_blit(path, &(&settings.pattern).into(), &split_settings, &mut cull_dummy);
}

pub fn pattern_along_glif<U: glifparser::PointData>(path: &Glif<U>, pattern: &Glif<U>, settings: &PatternSettings, marked_contour: Option<usize>) -> Glif<U>
{
    // convert our path and pattern to piecewise collections of beziers
    let piece_path = match path.outline {
        Some(ref o) => Piecewise::from(o),
        None => {return path.clone()}
    };
    let piece_pattern = Piecewise::from(pattern.outline.as_ref().unwrap());

    let mut output_outline: Outline<U> = Vec::new();
    let mut cull_cache = skia_safe::Path::new();

    for (idx, contour) in piece_path.segs.iter().enumerate() {
        // if we're only stroking a specific contour and this is not it we copy the existing pattern and return
        if let Some(specific_contour) = marked_contour {
            if idx != specific_contour {
                output_outline.push(contour.to_contour());
                continue;
            }
        }

        let result_outline = split_and_blit(contour, &piece_pattern, settings, &mut cull_cache);

        for result_contour in result_outline.segs
        {
            output_outline.push(result_contour.to_contour());
        }
    }

    return Glif {
        outline: Some(output_outline), 
        anchors: path.anchors.clone(),
        width: path.width,
        unicode: path.unicode.clone(),
        name: path.name.clone(),
        lib: Lib::None,
        components: path.components.clone(),
        guidelines: path.guidelines.clone(),
        images: path.images.clone(),
        note: path.note.clone(),
        filename: path.filename.clone(),
        ..Glif::default()
    };
}
