use super::{ArcLengthParameterization, Bezier, Evaluate, EvalScale, EvalTranslate, Parameterization, Piecewise, Vector, Rect};
use crate::vec2;

use glifparser::{Glif, Outline, glif::{PAPContour, PatternCopies, PatternSubdivide, PatternStretch}};
use skia_safe::{Path};

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
    pub cull_overlap: f64,
    pub two_pass_culling: bool,
    pub reverse_culling: bool,
    pub reverse_path: bool,
}

// This takes our pattern settings and translate/splits/etc our input pattern in preparation of the main algorithm. We prepare our input in 'curve space'. In this space 0 on the y-axis will fall onto a point on the path. A value greater or less than 0 represents offset
// vertically from the path. The x axis represents it's travel along the arclength of the path. Once this is done the main function can naively loop over all the Piecewises in the output
// vec without caring about any options except normal/tangent offset.
fn prepare_pattern(pattern: &Piecewise<Piecewise<Bezier>>, arclenparam: &ArcLengthParameterization, settings: &PatternSettings) 
    ->  Vec<Piecewise<Piecewise<Bezier>>>
{
    let mut output: Vec<Piecewise<Piecewise<Bezier>>> = Vec::new();
    
    // we clone our original pattern so we can work with it and have ownership
    // there is definitely a better faster way of doing this, but my rust knowledge is holding me back
    let mut working_pattern = pattern.translate(vec2!(1., 1.));
 
    let pattern_bounds = pattern.bounds();
    let pattern_width = f64::abs(pattern_bounds.left - pattern_bounds.right) * settings.pattern_scale.x;
    let pattern_height = f64::abs(pattern_bounds.bottom - pattern_bounds.top);

    // first order of business is to 0 out the pattern on the x axis and to put it's halfwidth at 0 on the y axis such that
    // the midpoint of this patterns bounding box lies at 0 on the y axis now we've got to calculate our pattern's bounds at this
    // point in the process the pattern is 0'd on the x axis
    if settings.center_pattern {
        let pattern_offset_x = -pattern_bounds.left as f64 - 1.;
        let pattern_offset_y = -pattern_bounds.bottom as f64 - 1.;

        working_pattern = working_pattern.translate(vec2!(pattern_offset_x, pattern_offset_y - pattern_height/2.));
        working_pattern = working_pattern.scale(vec2!(settings.pattern_scale.x, settings.pattern_scale.y));
    }

    // if we've got a simple split we just do that now 
    match settings.subdivide {
        PatternSubdivide::Simple(times) => {
            for _n in 0..times {
                working_pattern = working_pattern.subdivide(0.5);
            }
        }
        _ => {} // We're gonna handle the other options later in the process.
    }

    // so first up let's take our path and calculate how many patterns can fit along it
    // the last element of arclenparam gives us the total arc len over the entire path
    let total_arclen = arclenparam.get_total_arclen();


    // we add the width of the pattern and the spacing setting which gives us the overall width of each input pattern including the space
    // at it's end
    let total_width = pattern_width + settings.spacing;

    match settings.copies {
        PatternCopies::Single => {
            // if we have the stretch option enabled we respect the spacing setting, otherwise it doesn't really
            // make sense for a single copy
            let mut single_width = total_width;
            if settings.stretch == PatternStretch::On { single_width = pattern_width }

            // can we fit a copy of our pattern on this path?
            if f64::floor(total_arclen/single_width) > 0. {
                let mut single = working_pattern;

                if settings.stretch == PatternStretch::On {
                    let stretch_len = total_arclen - single_width;
                    single = single.scale(vec2!(1. + stretch_len/pattern_width, 1.));
                }

                output.push(single);
            }
        },

        PatternCopies::Repeated => {
            // we divide the total arc-length by our pattern's width and then floor it to the nearest integer
            // and this gives us the total amount of whole copies that could fit along this path
            let mut copies = (total_arclen/pattern_width) as i32;
            let mut left_over = total_arclen/pattern_width - copies as f64;
            let mut additional_spacing = 0.;

            while left_over < (settings.spacing / pattern_width) * (copies - 1) as f64 {
                copies -= 1;
                left_over = total_arclen/pattern_width - copies as f64;
            }
            left_over = left_over - (settings.spacing / pattern_width) * (copies - 1) as f64;

            let mut stretch_len = 0.;

            match settings.stretch {
                PatternStretch::On => {
                    // divide that by the number of copies and now we've got how much we should stretch each
                    stretch_len = left_over/copies as f64;
                    // now we divide the length by the pattern width and get a fraction which we add to scale
                    working_pattern = working_pattern.scale(vec2!(1. + stretch_len as f64, 1.));
                },
                PatternStretch::Spacing => {
                    // divide that by the number of copies and now we've got how much we should stretch each
                    additional_spacing = left_over/copies as f64 * pattern_width;
                },
                PatternStretch::Off => {}
            }

            for n in 0..copies {
                output.push(working_pattern.translate(vec2!(n as f64 * total_width + n as f64 * stretch_len * pattern_width + n as f64 * additional_spacing, 0.)));
            }
        }

        PatternCopies::Fixed(_n) => {
            // TODO: Implement
        }
    }

    return output;
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
fn pattern_along_path(path: &Piecewise<Bezier>, pattern: &Piecewise<Piecewise<Bezier>>, settings: &PatternSettings) -> Piecewise<Piecewise<Bezier>>
{
    // we're gonna parameterize the input path such that 0-1 = 0 -> totalArcLength
    // this is important because samples will be spaced equidistant along the input path
    let arclenparam = ArcLengthParameterization::from(path, 1000);
    let total_arclen = arclenparam.get_total_arclen();

    let mut output_segments: Vec<Piecewise<Bezier>> = Vec::new();

    let mut prepared_pattern = prepare_pattern(pattern, &arclenparam, settings);
    let pattern_bounds = pattern.bounds();
    let pattern_width = f64::abs(pattern_bounds.left - pattern_bounds.right) * settings.pattern_scale.x;

    let transform = |point: &Vector| {
        // if we're reversing the path we subtract u from 1 and get our reversed time
        let u = if settings.reverse_path { 1. - point.x/total_arclen } else { point.x/total_arclen };
    
        // Paramaterize u such that 0-1 maps to the curve by arclength
        let t = arclenparam.parameterize(u);
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
        return  P + path_point;
    };

    // This stores our cuts we identify during the culling process. If we have any cuts in this vec we're going
    // to call this function again with the cuts made if the two_pass setting is true.
    let mut cuts = Vec::new();

    let mut clipping_rects: Vec<Rect> = Vec::new();

    if settings.reverse_culling { prepared_pattern.reverse() };

    for p in prepared_pattern {
        let transformed_pattern = p.apply_transform(&transform);

        // TODO: Make this use convex hulls for more accuracy. Should be plenty fast enough, and handle our edge
        // cases a lot better. Use seperating axis theorem for collision detection. I'll need to implement quickhull, or
        // find an existing implementation. Current implementation has issues with stuff like long dashed lines on a diagonal
        // due to AABB

        // After we've transformed the patterns we're now going to get an axis-aligned bounding box, and
        // compare it to all the previous ones. If we find a % overlap by total bounding box area then we discard
        // this one and move on to the next. This is relatively naive culling, and could be improved
        // by turning the path into a fat polyline and clipping it against itself prior to preparing the pattern.
        // This would allow you to have correct and consistent spacing where the patterns are culled.
        if settings.cull_overlap != 1. {
            // we inflate this rect by the spacing around it
            let mut this_rect = transformed_pattern.bounds();

            let mut greatest_overlap = 0.;
            let mut overlap_index = 0;
            let mut overlapping_rect: Option<Rect> = None;
            for (i, rect) in clipping_rects.iter().enumerate() {
                if this_rect.overlaps(rect) {
                    if this_rect.overlap_rect(rect).area() > greatest_overlap {
                        greatest_overlap = this_rect.overlap_rect(&rect).area();
                        overlapping_rect = Some(rect.clone());
                        overlap_index = i;
                    }
                }
            }

            if let Some(rect) = overlapping_rect {
                // we found an overlap now we need to get the percentage of the
                // area of the overlap
                let area_of_overlap = this_rect.overlap_rect(&rect).area();
                let total_area = this_rect.area() + rect.area();

                let fractional_overlap = (area_of_overlap * 2.) / total_area;

                // if we're checking against the preceding pattern we nudge the overlap towards non-collision
                // we are inflating the pattern's AABBs by the spacing in each direction, so if it's overlapping
                // less than that or equal to that we don't want to discard
                let nudging = if overlap_index == clipping_rects.len() - 1 { settings.spacing / total_area.sqrt() } else { 0. };
                if fractional_overlap - nudging > settings.cull_overlap {
                    let start_len = p.bounds().left;
                    let end_len = p.bounds().right;

                    // we push our cuts to the cut vec one at the start and end of the pattern plus the configured spacing
                    cuts.push(arclenparam.parameterize(start_len / total_arclen));
                    cuts.push(arclenparam.parameterize(end_len / total_arclen));

                    // if our percentage overlap is greater than the setting we're going to drop this instance of the pattern pattern
                    continue;
                }
            }

            this_rect.left = this_rect.left - settings.spacing;
            this_rect.right = this_rect.right + settings.spacing;
            this_rect.bottom = this_rect.bottom - settings.spacing;
            this_rect.top = this_rect.top + settings.spacing;
            clipping_rects.push(this_rect);
        }

        for contour in transformed_pattern.segs {
            output_segments.push(contour);
        }
    }

    // if we have a cut and we have two pass culling enabled we recursively call this function
    // after splitting the paths at our best guess of the collisions we found in the culling
    // step 
    if !cuts.is_empty() && settings.two_pass_culling {
        let mut new_settings = settings.clone();
        new_settings.cull_overlap = 1.; // we copy our settings but set overlap to false so we don't do this more than once.
        new_settings.two_pass_culling = false;

        // Clone our path and make our cuts. We might want to re-parameterize between these cuts to keep their location consistent.
        let mut new_path = path.clone();
        for cut in &cuts {
            let cut_path = new_path.cut_at_t(*cut);
            new_path = cut_path;
        }

        let trimmed_path = new_path.remove_short_segs(pattern_width * 2. + settings.spacing, 100);
        let split_path = trimmed_path.split_at_discontinuities(0.01);
        let mut output = Vec::new();
        for sub_path in split_path.segs {
            let second_path = pattern_along_path(&sub_path, pattern, &new_settings);
            for contour in second_path.segs {
                output.push(contour);
            }
        }

        return Piecewise::new(output, None);
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
    };

    pattern_along_path(path, &(&settings.pattern).into(), &split_settings)
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

    for (idx, contour) in piece_path.segs.iter().enumerate() {
        // if we're only stroking a specific contour and this is not it we copy the existing pattern and return
        if let Some(specific_contour) = marked_contour {
            if idx != specific_contour {
                output_outline.push(contour.to_contour());
                continue;
            }
        }

        let mut result_pw = pattern_along_path(&contour, &piece_pattern, settings);

        if settings.simplify {
            let skpattern: Path = result_pw.to_skpath();
            result_pw = Piecewise::from(&skpattern.simplify().unwrap().as_winding().unwrap());
        }

        let result_outline = result_pw.to_outline();

        for result_contour in result_outline
        {
            output_outline.push(result_contour);
        }
    }

    return Glif {
        outline: Some(output_outline), 
        order: path.order, // default when only corners
        anchors: path.anchors.clone(),
        width: path.width,
        unicode: path.unicode.clone(),
        name: path.name.clone(),
        lib: None,
        components: path.components.clone(),
        guidelines: path.guidelines.clone(),
        images: path.images.clone(),
        note: path.note.clone(),
        filename: path.filename.clone(),
        ..Glif::default()
    };
}
