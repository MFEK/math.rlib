use std::f64::consts;

use super::{Bezier, Evaluate, Piecewise, Vector, EvalScale, EvalTranslate, EvalRotate};
use super::piecewise::glif::PointData;
use glifparser::{Glif};

use flo_curves::line::{line_intersects_line};
use crate::{consts::SMALL_DISTANCE, coordinate::Coordinate, vec2};


fn vec2_to_rad(vec: Vector) -> f64
{
    return f64::atan2(vec.y, vec.x);
}

fn normalize_angle(angle: f64) -> f64
{
    let mut angle = angle;
    while angle < 0. { angle += consts::TAU }
    while angle > consts::TAU { angle -= consts::TAU }

    angle
}
fn delta_angle(start: f64, end: f64, direction: f64) -> f64
{
    let mut difference = if direction == 1. {
        start - end
    } else {
        end - start
    };


    difference
}

pub struct GlyphBuilder {
    pub beziers: Vec<Bezier>
}

impl GlyphBuilder {
    pub fn new() -> Self {
        return Self {
            beziers: Vec::new()
        }
    }

    pub fn append(&mut self, other: GlyphBuilder)
    {
        for bezier in other.beziers {
            self.bezier_to(bezier);
        }
    }

    pub fn append_vec(&mut self, other: Vec<Bezier>)
    {
        for bezier in other {
            self.bezier_to(bezier);
        }
    }

    pub fn bezier_to(&mut self, bez: Bezier)
    {
        self.beziers.push(bez);
    }

    // need to mvoe this stuff to it's own struct or use flo_curves PathBuilder
    pub fn line_to(&mut self, to: Vector)
    {
        let from = self.beziers.last().unwrap().end_point();
        let line = Bezier::from_points(from, from, to, to);

        self.beziers.push(line);
    }

    pub fn bevel_to(&mut self, to: Vector, _tangent1: Vector, _tangent2: Vector)
    {
        return self.line_to(to);
    }

    pub fn miter_to(&mut self, to: Vector, tangent1: Vector, tangent2: Vector)
    {
        let from = self.beziers.last().unwrap().end_point();
        let _intersection = Self::find_discontinuity_intersection(from, to, tangent1, -tangent2);


        if let Some(intersection) = _intersection {
            // found an intersection so we draw a line to it
            if from.distance(intersection) < from.distance(to) {  self.line_to(intersection);}
            self.line_to(to);
        }
        else
        {
            // if no intersection can be found we default to a bevel
            self.line_to(to);
        }
    }

    // https://www.stat.auckland.ac.nz/~paul/Reports/VWline/line-styles/line-styles.html
    pub fn arc_to(&mut self, to: Vector, tangent1: Vector, tangent2: Vector)
    {
        let from = self.beziers.last().unwrap().end_point();
        let dot_product = tangent1.dot(tangent2);
        let angle = f64::acos(dot_product);

        let n = f64::abs(consts::TAU/angle);
        let tangent1_right = Vector{x: tangent1.y, y: -tangent1.x}.normalize();
        let tangent2_right = Vector{x: tangent2.y, y: -tangent2.x}.normalize();

        let line1 = (from, from + tangent1_right * 2048.);
        let line2 = (to, to + tangent2_right * 2048.);

        // we shoot rays from the right of both of the tangents to find a center of the circle we're
        // going to make an arc of
        let intersection =  line_intersects_line(&line1, &line2);

        let circle_center = match intersection {
            Some(circle_center) => { 
                // if the center is very far away or if the tangents are parallel we discard any intersections
                if circle_center.distance(from) > from.distance(to)*2. { from.lerp(to, 0.5) }
                else if tangent1.distance(-tangent2) < SMALL_DISTANCE { from.lerp(to, 0.5) }
                else if tangent1.distance(tangent2) < SMALL_DISTANCE { from.lerp(to, 0.5) }
                else { circle_center }
            }
            None =>{ from.lerp(to, 0.5) } 
        };

        let radius = from.distance(circle_center);
        let dist_along_tangents = radius*(4./3.)*f64::tan(consts::PI/(2. * n));

        let arc = Bezier::from_points(
            from,
            from + tangent1 * dist_along_tangents,
            to + -tangent2 * dist_along_tangents,
            to
        );
        self.bezier_to(arc);
    }

    pub fn circle_arc_to(&mut self, to: Vector, tangent1: Vector, tangent2: Vector)
    {
        let from = self.beziers.last().unwrap().end_point();

        let tangent1_right = Vector{x: tangent1.y, y: -tangent1.x}.normalize();
        let tangent2_right = Vector{x: tangent2.y, y: -tangent2.x}.normalize();

        let ray1 = (from, from + tangent1_right*2048.);
        let ray2 = (to, to + tangent2_right*2048.);

        // we shoot rays from the right of both of the tangents to find a center of the circle we're
        // going to make an arc of
        let intersection =  line_intersects_line(&ray1, &ray2);

        let circle_center = match intersection {
            Some(circle_center) => { 
                if tangent1.distance(-tangent2) < SMALL_DISTANCE{
                    from.lerp(to, 0.5)
                } else{
                    circle_center 
                }
            }
            None =>{
                from.lerp(to, 0.5)
            } 
        };

        let radius = from.distance(circle_center);

        // the right vector is the product of an angle of 0 through (cos(angle), sin(angle)).
        // we find the difference between right and our midpoint -> from vector to figure out our
        // starting angle on the circle
        let degrees_90 = f64::to_radians(90.);
        //let right = vec2!(1., 0.);
        let starting_angle = normalize_angle(f64::atan2(tangent1_right.y, tangent1_right.x));
        let ending_angle = normalize_angle(f64::atan2(tangent2_right.y, tangent2_right.x));

        let total_angle = starting_angle - ending_angle;
        let repetitions = f64::floor(f64::abs(total_angle) / degrees_90);

        let mut first_point = true;
        for n in 0 .. repetitions as usize {
            let cur_angle = starting_angle + degrees_90 * n as f64;
            let next_angle = cur_angle + degrees_90;
            let cp1 = vec2!(f64::cos(cur_angle), f64::sin(cur_angle)) * radius + circle_center;

            if first_point { 
                let last = self.beziers.pop().unwrap();
                self.beziers.push(Bezier::from_points(last.w1, last.w2, last.w3, cp1));
            }

            let cp2 = vec2!(f64::cos(next_angle), f64::sin(next_angle)) * radius + circle_center;

            let handle_distance = (4./3.)*f64::tan(consts::PI/8.);
            let circle_tangent1 = -Vector{x: f64::sin(cur_angle), y: -f64::cos(cur_angle)}.normalize();
            let circle_tangent2 = Vector{x: f64::sin(next_angle), y: -f64::cos(next_angle)}.normalize();

            let h1 = cp1 + circle_tangent1 * radius * handle_distance;
            let h2 = cp2 + circle_tangent2 * radius * handle_distance;
            let circle_segment = Bezier::from_points(cp1, h1, h2, cp2);

            self.bezier_to(circle_segment);
            first_point = false;
        }

        let last_angle = starting_angle + degrees_90 * repetitions;
        let difference = f64::atan2(f64::sin(last_angle-ending_angle), f64::cos(last_angle-ending_angle));
        let n = f64::abs((consts::PI * 2.) / difference);

        if f64::abs(difference) < 0.1 { 
            // we've gotta make sure out last point is lined up with to
            let last_bez = self.beziers.pop();
            let cps = last_bez.unwrap().to_control_points();

            self.bezier_to(Bezier::from_points(cps[0], cps[1], cps[2], to));
        };

        let cp1 = vec2!(f64::cos(last_angle), f64::sin(last_angle)) * radius + circle_center;
        let cp2 = to;

        let handle_distance = (4./3.)*f64::tan(consts::PI/(2.*n));
        let circle_tangent1 = -Vector{x: f64::sin(last_angle), y: -f64::cos(last_angle)}.normalize();
        let circle_tangent2 = Vector{x: f64::sin(ending_angle), y: -f64::cos(ending_angle)}.normalize();
        let h1 = cp1 + circle_tangent1 * radius * handle_distance;
        let h2 = cp2 + circle_tangent2 * radius * handle_distance;

        let circle_segment = Bezier::from_points(cp1, h1, h2, cp2);

        self.bezier_to(circle_segment);
    }

    pub fn cap_to(&mut self, to: Vector, cap: &Glif<Option<PointData>>)
    {
        let cap_pw = Piecewise::from(cap.outline.as_ref().unwrap().first().unwrap());
        let from = self.beziers.last().unwrap().end_point();
        let join_mid_point = from.lerp(to, 0.5);
        
        let cap_first_point = cap_pw.segs.first().unwrap().start_point();
        let cap_last_point = cap_pw.segs.last().unwrap().end_point();

        // get the distance from -> to and use that to scale the cap
        let goal_size = from.distance(to);
        let cur_size = cap_first_point.distance(cap_last_point);
        let scaled_cap = cap_pw.scale(vec2!(goal_size/cur_size, goal_size/cur_size));

        // we need to center the cap at the point between the first point in the contour and the last
        let s_cap_first_point = scaled_cap.segs.first().unwrap().start_point();
        let s_cap_last_point = scaled_cap.segs.last().unwrap().end_point();
        let cap_mid_point = s_cap_first_point.lerp(s_cap_last_point, 0.5);
        // we translate by -mid_point which brings the cap's midpoint to origin
        let translated_cap = scaled_cap.translate(vec2!(-cap_mid_point.x, -cap_mid_point.y));

        // then we rotate the cap into position by getting the angle between +1, 0 and the normalized tangent
        // of the line between from->to and rotating it
        let tangent = from - to;
        let cap_tangent = s_cap_first_point - s_cap_last_point;
        let normal = vec2!(tangent.y, -tangent.x).normalize();
        let cap_normal = vec2!(cap_tangent.y, -cap_tangent.x).normalize();

        let dot_product = normal.dot(-cap_normal);
        let angle = {
            let a = f64::acos(dot_product);
            if normal.y.is_sign_negative() { -a } else { a }
        };

        let rotated_cap = translated_cap.rotate(angle);

        // finally we translate the cap from 0,0 being the midpoint between the caps first and last points
        // to the mid point between from and to moving from 'cap space' to 'world space'
        let final_cap = rotated_cap.translate(join_mid_point);

        for bezier in final_cap.segs.iter().rev() {
            self.bezier_to(bezier.reverse());
        }
    }

    
    fn find_discontinuity_intersection(from: Vector, to: Vector, tangent1: Vector, tangent2: Vector) -> Option<Vector>
    {
        // create rays starting at from and to and pointing in the direction of the respective tangent
        let ray1 = (from, from + tangent1*200.);
        let ray2 = (to, to + tangent2*200.);

        return line_intersects_line(&ray1, &ray2);
    }

    pub fn fuse_nearby_ends(&self, distance: f64) -> GlyphBuilder {
        let mut iter = self.beziers.iter().peekable();
        let mut new_segments = Vec::new();
        while let Some(primitive) = iter.next() {
            if let Some(next_primitive) = iter.peek() {
                if primitive.end_point().distance(next_primitive.start_point()) <= distance {
                    let new_primitive = primitive.to_control_points();
                    new_segments.push(Bezier::from_points(new_primitive[0], new_primitive[1], new_primitive[2], next_primitive.start_point()));
                    continue;
                }
            }

            new_segments.push(primitive.clone());
        }

        return GlyphBuilder{ beziers: new_segments };
    }
    
}
