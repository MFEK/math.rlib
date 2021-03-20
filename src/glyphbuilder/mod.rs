use super::{Bezier, Evaluate, Piecewise, Vector, EvalScale, EvalTranslate, EvalRotate};
use super::piecewise::glif::PointData;
use glifparser::{Glif};

use flo_curves::line::{line_intersects_line};
use crate::vec2;

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
        let _intersection = Self::find_discontinuity_intersection(from, to, tangent1, tangent2);


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
        let _intersection = Self::find_discontinuity_intersection(from, to, tangent1, tangent2);
        
        if let Some(intersection) = _intersection {
            if intersection.distance(from) < from.distance(to)
            {
                let radius = f64::min(from.distance(intersection), to.distance(intersection));
                let angle = f64::acos(from.dot(to) / (from.magnitude() * to.magnitude()));
                let dist_along_tangents = radius*(4./(3.*(1./f64::cos(angle/2.) + 1.)));

                let arc = Bezier::from_points(from, from + tangent1 * dist_along_tangents, to + tangent2 * dist_along_tangents, to);
                self.bezier_to(arc);
            }
            else
            {
                let radius = from.distance(to) * (2./3.);
                let angle = f64::acos(from.dot(to) / (from.magnitude() * to.magnitude()));
                let dist_along_tangents = radius*(4./(3.*(1./f64::cos(angle/2.) + 1.)));
    
                let arc = Bezier::from_points(
                    from,
                    from + tangent1 * dist_along_tangents,
                    to + tangent2 * dist_along_tangents,
                    to
                );
                self.bezier_to(arc);
            }
        }
        else
        {
            let radius = from.distance(to) * (2./3.);
            let angle = f64::acos(from.dot(to) / (from.magnitude() * to.magnitude()));
            let dist_along_tangents = radius*(4./(3.*(1./f64::cos(angle/2.) + 1.)));

            let arc = Bezier::from_points(
                from,
                from + tangent1 * dist_along_tangents,
                to + tangent2 * dist_along_tangents,
                to
            );
            self.bezier_to(arc);
        }
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

        let angle = f64::acos(normal.dot(-cap_normal));
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

}
