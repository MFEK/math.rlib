use super::{Bezier, Evaluate, Piecewise, Vector, GlyphBuilder};
use super::consts::{SMALL_DISTANCE};
use glifparser::{Glif, JoinType, Outline, VWSContour, glif::{CapType, InterpolationType, MFEKPointData, VWSHandle}};

#[derive(Debug)]
pub struct VWSSettings {
    pub cap_custom_start: Option<Glif<MFEKPointData>>,
    pub cap_custom_end: Option<Glif<MFEKPointData>>,
}

// we want to deal with colocated handles here so that we don't get funky results at caps and joins
// where one or more handles is colocated
fn preprocess_path(in_pw: &Piecewise<Bezier>) -> Piecewise<Bezier>
{
    let in_pw = in_pw.remove_short_segs(1., 3);
    let mut out_contours = Vec::new();

    for bez in &in_pw.segs {
        let mut new_bez = bez.clone();

        let distance_heuristic = bez.w1.distance(bez.w4)/12.;
        if bez.w1.distance(bez.w2) < distance_heuristic {
            new_bez.w2 = bez.at(0.40);
        }

        if bez.w3.distance(bez.w4) < distance_heuristic {
            new_bez.w3 = bez.at(0.60);
        }

        out_contours.push(new_bez);
    }

    return Piecewise {
        segs: out_contours,
        cuts: in_pw.cuts.clone()
    }
}
// takes a vector of beziers and fills in discontinuities with joins
fn fix_path(in_path: GlyphBuilder, closed: bool, join_type: JoinType) -> GlyphBuilder
{
    let mut out = GlyphBuilder::new();

    let join_to = match join_type {
        JoinType::Bevel => GlyphBuilder::bevel_to,
        JoinType::Round => GlyphBuilder::arc_to,
        JoinType::Circle => GlyphBuilder::circle_arc_to,
        JoinType::Miter => GlyphBuilder::miter_to
    };

    let mut path_iter = in_path.beziers.iter().peekable();
    
    while let Some(bezier) = path_iter.next() {
        if let Some(next_bezier) = path_iter.peek()
        {
            let next_start = next_bezier.start_point();
            let last_end = bezier.end_point();
            if !last_end.is_near(next_start, SMALL_DISTANCE)
            {
                // the end of our last curve doesn't match up with the start of our next so we need to
                // deal with the discontinuity be creating a join
                let from_end_point = bezier.at(1.);
                let to_start_point = next_bezier.at(0.);
        
                // used for round joins
                let tangent1 = bezier.tangent_at(1.).normalize(); 
                let tangent2 = next_bezier.tangent_at(0.).normalize();
        
                let discontinuity_vec = to_start_point - from_end_point;
                let dr =  Vector{x: discontinuity_vec.y, y: -discontinuity_vec.x}.normalize();
        
                let t1_dot_dr = tangent1.dot(dr);
                let t2_dot_dr = tangent2.dot(dr);
                let tangent1= if t1_dot_dr > 0. { tangent1 } else { -tangent1 };
                let tangent2= if t2_dot_dr < 0. { tangent2 } else { -tangent2 };
    
                
                out.bezier_to(bezier.clone());
                join_to(&mut out, next_start, tangent1, tangent2);
            }
            else
            {
                out.bezier_to(bezier.clone());
            }
        }
        else if closed
        {
            // our path is closed and if there's not a next point we need to make sure that our current
            // and last curve matches up with the first one

            let first_bez = in_path.beziers.first().unwrap();
            let first_point = first_bez.start_point();
            let last_end = bezier.end_point();

            if !last_end.is_near(first_point, SMALL_DISTANCE)
            {
                let tangent1 = bezier.tangent_at(1.).normalize(); 
                let tangent2 = first_bez.tangent_at(0.).normalize();
                let discontinuity_vec = first_point - last_end;
                let on_outside = Vector::dot(tangent2, discontinuity_vec) <= 0.;

                if !on_outside {
                    out.bezier_to(bezier.clone());
                    join_to(&mut out, first_point, tangent1, tangent2);
                }
                else
                {
                    out.bezier_to(bezier.clone());
                    out.line_to(first_point);
                }
            }
            else
            {
                out.bezier_to(bezier.clone());
            }
        }
        else
        {
            out.bezier_to(bezier.clone());
        }
    }

    return out;
}

pub fn variable_width_stroke(in_pw: &Piecewise<Bezier>, vws_contour: &VWSContour, settings: &VWSSettings) -> Piecewise<Piecewise<Bezier>> {
    let in_pw = preprocess_path(in_pw);

    let closed = in_pw.is_closed();
    let stroke_handles = &vws_contour.handles;

    // check if our input path is closed
    // We're gonna keep track of a left line and a right line.
    let mut left_line = GlyphBuilder::new();
    let mut right_line = GlyphBuilder::new();

    let iter = in_pw.segs.iter().enumerate();
    for (i, bezier) in iter {
        let cur_handle = &stroke_handles[i];
        let next_handle = &stroke_handles[i+1];

        let left_start = cur_handle.left_offset;
        let right_start = cur_handle.right_offset;

        let left_end = match cur_handle.interpolation {
            InterpolationType::Null => left_start,
            _ => next_handle.left_offset
        };

        
        let right_end = match cur_handle.interpolation {
            InterpolationType::Null => right_start,
            _ => next_handle.right_offset
        };

        let max_tangent_start = f64::max(cur_handle.right_offset, cur_handle.left_offset);
        let max_tangent_end = f64::max(next_handle.right_offset, next_handle.left_offset);

        let left_ratio_start = cur_handle.left_offset/max_tangent_start;
        let left_ratio_end = next_handle.left_offset/max_tangent_end;

        let right_ratio_start = cur_handle.right_offset/max_tangent_start;
        let right_ratio_end = next_handle.right_offset/max_tangent_end;

        let tangent_start = cur_handle.tangent_offset;
        let tangent_end = next_handle.tangent_offset;

        let calc_t2 = |t| (1.-f64::cos(t*std::f64::consts::PI))/2.;

        let left_normal_closure = |t| {
            let t2 = calc_t2(t);
            return -left_start*(1.-t2)+-left_end*t2
        };

        let left_tangent_closure = |t| {
            let t2 = calc_t2(t);
            return -tangent_start*left_ratio_start*(1.-t2)+-tangent_end*left_ratio_end*t2
        };
        
        let right_normal_closure = |t| {
            let t2 = calc_t2(t);
            return right_start*(1.-t2)+right_end*t2
        };

        let right_tangent_closure = |t| {
            let t2 = calc_t2(t);
            return tangent_start*right_ratio_start*(1.-t2)+tangent_end*right_ratio_end*t2
        };

        let left_offset = flo_curves::bezier::offset_lms_sampling(bezier, left_normal_closure, left_tangent_closure, 20, 4.0);
        left_line.append_vec(left_offset.unwrap());

        let right_offset = flo_curves::bezier::offset_lms_sampling(bezier, right_normal_closure, right_tangent_closure, 20, 4.0);
        right_line.append_vec(right_offset.unwrap());
    }
     
    right_line.beziers.reverse();
    right_line = GlyphBuilder {
        beziers:    right_line.beziers.iter()
                    .map(|bez| bez.clone().reverse())
                    .collect()
    };

    right_line = right_line.fuse_nearby_ends(0.01);
    left_line = left_line.fuse_nearby_ends(0.01);

    right_line = fix_path(right_line, closed, vws_contour.join_type);
    left_line = fix_path(left_line, closed, vws_contour.join_type);

    if in_pw.is_closed() {
        let mut out = Vec::new();

        let left_pw = Piecewise::new(left_line.beziers, None);
        let right_pw = Piecewise::new(right_line.beziers, None);

        if !vws_contour.remove_internal {
            out.push(left_pw);
        }
        if !vws_contour.remove_external {
            out.push(right_pw);
        }
        
        return Piecewise::new(out, None);
    }
    else
    {
        // path is not closed we need to cap the ends
        let mut out_builder = left_line;

        let from = out_builder.beziers.last().unwrap().clone();
        let to = right_line.beziers.first().unwrap().clone();
        
        let from_end_point = from.at(1.);
        let to_start_point = to.at(0.);

        // used for round joins
        let tangent1 = from.tangent_at(1.).normalize(); 
        let tangent2 = -to.tangent_at(0.).normalize();

        let discontinuity_vec = to_start_point - from_end_point;
        let dr =  Vector{x: discontinuity_vec.y, y: -discontinuity_vec.x}.normalize();

        let tangent1= if tangent1.dot(dr) < 0.9 { dr } else { tangent1 };
        let tangent2= if tangent2.dot(dr) > -0.9 { -dr } else { tangent2 };

        match vws_contour.cap_end_type {
            CapType::Round => out_builder.arc_to(to.start_point(), tangent1, tangent2),
            CapType::Circle => out_builder.circle_arc_to(to.start_point(), tangent1, tangent2),
            CapType::Square => out_builder.line_to(to.start_point()),
            CapType::Custom => out_builder.cap_to(to.start_point(), settings.cap_custom_end.as_ref().unwrap())
        }

        // append the right line to the left now that we've connected them
        out_builder.append(right_line);

        // we need to close the beginning now 
        let from = out_builder.beziers.last().unwrap().clone();
        let to = out_builder.beziers.first().unwrap().clone();

        let from_end_point = from.at(1.);
        let to_start_point = to.at(0.);

        // used for round joins
        let tangent1 = from.tangent_at(1.).normalize(); 
        let tangent2 = -to.tangent_at(0.).normalize();

        let discontinuity_vec = to_start_point - from_end_point;
        let dr =  Vector{x: discontinuity_vec.y, y: -discontinuity_vec.x}.normalize();

        let tangent1= if tangent1.dot(dr) < 0.9 { dr } else { tangent1 };
        let tangent2= if tangent2.dot(dr) > -0.9 { -dr } else { tangent2 };

        match vws_contour.cap_start_type {
            CapType::Round => out_builder.arc_to(to.start_point(), tangent1, tangent2),
            CapType::Circle => out_builder.circle_arc_to(to.start_point(), tangent1, tangent2),
            CapType::Square => out_builder.line_to(to.start_point()),
            CapType::Custom => out_builder.cap_to(to.start_point(), settings.cap_custom_start.as_ref().unwrap())
        }

        let inner = Piecewise::new(out_builder.beziers, None);
        return Piecewise::new(vec![inner], None);
    } 

}

pub fn variable_width_stroke_glif<U: glifparser::PointData>(path: &Glif<U>, settings: VWSSettings) -> Glif<MFEKPointData>
{
    // convert our path and pattern to piecewise collections of beziers
    let piece_path = Piecewise::from(path.outline.as_ref().unwrap());
    let mut output_outline: Outline<MFEKPointData> = Vec::new();

    let handles = parse_vws_lib(path);

    if handles.is_none() {
        panic!("No vws contours found in input!")
    }
    
    let handles = handles.expect("Input glyph has no lib node!");

    let iter = piece_path.segs.iter().enumerate();
    for (i, pwpath_contour) in iter {
        let vws_contour = find_vws_contour(i, &handles.0);
        
        if let Some(contour) = vws_contour {
            let results = variable_width_stroke(&pwpath_contour, &contour, &settings);
            for result_contour in results.segs {
                output_outline.push(result_contour.to_contour());
            }
        }
        else
        {
            output_outline.push(pwpath_contour.to_contour());
        }
    }
    
    return Glif {
        outline: Some(output_outline),
        order: path.order, // default when only corners
        anchors: path.anchors.clone(),
        width: path.width,
        unicode: path.unicode.clone(),
        name: path.name.clone(),
        format: 2,
        lib: Some(handles.1),
        components: path.components.clone(),
        guidelines: path.guidelines.clone(),
        images: path.images.clone(),
        note: path.note.clone(),
        filename: path.filename.clone(),
        private_lib: path.private_lib.clone(),
        ..Glif::default()
    };
}

pub fn find_vws_contour(id: usize, vws_outline: &Vec<VWSContour>) -> Option<&VWSContour>
{
    return vws_outline.get(id);
}

pub fn parse_vws_lib<T: glifparser::PointData>(input: &Glif<T>) -> Option<(Vec<VWSContour>, xmltree::Element)>
{
    if let Some(lib) = input.lib.as_ref() {
        let mut lib = lib.clone();
        let mut vws_outline = Vec::new();

        while let Some(mut vws) = lib.take_child("vws") {
            let _name = vws
                .attributes
                .get("id")
                .expect("VWSContour must have an id");

            let cap_start = vws
                .attributes
                .get("cap_start");
            
            let cap_end = vws
                .attributes
                .get("cap_end");
            
            let join = vws
                .attributes
                .get("join");

            let round_str ="round".to_string();
            let cap_start = cap_start.unwrap_or(&round_str);
            let cap_end = cap_end.unwrap_or(&round_str);
            let join = join.unwrap_or(&round_str);

            let cap_start_type = match cap_start.as_str() {
                "round" => CapType::Round,
                "circle" => CapType::Circle,
                "square" => CapType::Square,
                "custom" => CapType::Custom,
                _ => panic!("Invalid cap type!")
            };
            
            let cap_end_type = match cap_end.as_str() {
                "round" => CapType::Round,
                "circle" => CapType::Circle,
                "square" => CapType::Square,
                "custom" => CapType::Custom,
                _ => panic!("Invalid cap type!")
            };

            let join_type = match join.as_str() {
                "round" => JoinType::Round,
                "circle" => JoinType::Circle,
                "miter" => JoinType::Miter,
                "bevel" => JoinType::Bevel,
                _ => panic!("Invalid join type!")
            };

            let mut vws_handles = VWSContour {
                handles: Vec::new(),
                cap_start_type: cap_start_type,
                cap_end_type: cap_end_type,
                join_type: join_type,
                remove_internal: false, // TODO: Add these to <lib>
                remove_external: false,
            };

            while let Some(vws_handle) = vws.take_child("handle") {
                let left: f64 = vws_handle
                    .attributes
                    .get("left")
                    .expect("VWSHandle missing left")
                    .parse()
                    .expect("VWSHandle not float.");

                let right: f64 = vws_handle
                    .attributes
                    .get("right")
                    .expect("VWSHandle missing right")
                    .parse()
                    .expect("VWSHandle not float.");

                let tangent: f64 = vws_handle
                    .attributes
                    .get("tangent")
                    .expect("VWSHandle missing tangent")
                    .parse()
                    .expect("VWSHandle tangent not float.");

                let interpolation_string: &String = vws_handle
                    .attributes
                    .get("interpolation")
                    .expect("VWSHandle missing interpolation type");
                    

                let interpolation = match interpolation_string.as_str() {
                    "linear" => InterpolationType::Linear,
                    _ => InterpolationType::Null
                };

                vws_handles.handles.push(VWSHandle{
                    left_offset: left,
                    right_offset: right,
                    tangent_offset: tangent,
                    interpolation: interpolation
                });
            }

            vws_outline.push(vws_handles);
        }

        if vws_outline.len() > 0 {
            return Some((vws_outline, lib));
        }
    }

    return None;
}

pub fn cap_type_to_string(ct: CapType)  -> String
{
    match ct {
        CapType::Round => "round".to_string(),
        CapType::Circle => "circle".to_string(),
        CapType::Square => "square".to_string(),
        CapType::Custom => "custom".to_string(),
    }
}

pub fn join_type_to_string(jt: JoinType)  -> String
{
    match jt {
        JoinType::Round => "round".to_string(),
        JoinType::Circle => "circle".to_string(),
        JoinType::Miter => "miter".to_string(),
        JoinType::Bevel => "bevel".to_string(),
    }
}

pub fn generate_vws_lib(vwscontours:  &Vec<VWSContour>) -> Option<xmltree::Element>
{
    if vwscontours.len() == 0 { return None }
    let mut lib_node = xmltree::Element::new("lib");

    for vwcontour in vwscontours {
        let mut vws_node = xmltree::Element::new("vws");
         vws_node.attributes.insert("cap_start".to_owned(), cap_type_to_string(vwcontour.cap_start_type));
         vws_node.attributes.insert("cap_end".to_owned(), cap_type_to_string(vwcontour.cap_end_type));
         vws_node.attributes.insert("join".to_owned(), join_type_to_string(vwcontour.join_type));

        for handle in &vwcontour.handles {
            let mut handle_node = xmltree::Element::new("handle");
            handle_node.attributes.insert("left".to_owned(), handle.left_offset.to_string());
            handle_node.attributes.insert("right".to_owned(), handle.right_offset.to_string());
            handle_node.attributes.insert("tangent".to_owned(), handle.tangent_offset.to_string());


            match handle.interpolation {
                InterpolationType::Linear => {handle_node.attributes.insert("interpolation".to_owned(), "linear".to_owned());},
                InterpolationType::Null => {handle_node.attributes.insert("interpolation".to_owned(), "none".to_owned());}
            }
            
            vws_node.children.push(xmltree::XMLNode::Element(handle_node));
        }

        lib_node.children.push(xmltree::XMLNode::Element(vws_node));
    }

    return Some(lib_node);
}
