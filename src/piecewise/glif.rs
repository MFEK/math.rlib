use super::{Bezier, Outline, Piecewise, Vector};
use glifparser::{Contour, Handle, PointType, glif::{MFEKContour, MFEKOutline, MFEKPointData}};
use super::super::consts::SMALL_DISTANCE;

impl<T: glifparser::PointData> From<&Outline<T>> for Piecewise<Piecewise<Bezier>>
{
    fn from(outline: &Outline<T>) -> Self { 
        let mut new_segs = Vec::new();

        for contour in outline
        {
            new_segs.push(Piecewise::from(contour));
        }
    
        return Piecewise::new(new_segs, None);
    }
}

impl<T: glifparser::PointData> From<&MFEKOutline<T>> for Piecewise<Piecewise<Bezier>>
{
    fn from(outline: &MFEKOutline<T>) -> Self { 
        let mut new_segs = Vec::new();

        for contour in outline
        {
            new_segs.push(Piecewise::from(contour));
        }
    
        return Piecewise::new(new_segs, None);
    }
}

impl<T: glifparser::PointData> From<MFEKOutline<T>> for Piecewise<Piecewise<Bezier>>
{
    fn from(outline: MFEKOutline<T>) -> Self { 
        return outline.into();
    }
}

impl Piecewise<Piecewise<Bezier>> {
    pub fn to_outline(&self) -> Outline<MFEKPointData> {
        let mut output_outline: Outline<MFEKPointData> = Outline::new();

        for contour in &self.segs
        {
            output_outline.push(contour.to_contour());
        }

        return output_outline;
    }
}

impl<T: glifparser::PointData> From<&Contour<T>> for Piecewise<Bezier>
{
    fn from(contour: &Contour<T>) -> Self {
        let mut new_segs = Vec::new();

        let mut lastpoint: Option<&glifparser::Point<T>> = None;

        for point in contour
        {
            match lastpoint
            {
                None => {},
                Some(lastpoint) => {
                    new_segs.push(Bezier::from(&lastpoint, point));
                }
            }

            lastpoint = Some(point);
        }

        let firstpoint = contour.first().unwrap();
        if firstpoint.ptype != PointType::Move {
            new_segs.push(Bezier::from(&lastpoint.unwrap(), firstpoint));
        }

        return Piecewise::new(new_segs, None);
    }
}

impl<T: glifparser::PointData> From<&MFEKContour<T>> for Piecewise<Bezier>
{
    fn from(contour: &MFEKContour<T>) -> Self {
        return Piecewise::from(&contour.inner);
    }
}

impl<T: glifparser::PointData> From<MFEKContour<T>> for Piecewise<Bezier>
{
    fn from(contour: MFEKContour<T>) -> Self {
        return Piecewise::from(&contour.inner);
    }
}

impl Piecewise<Bezier> {
    pub fn to_contour(&self) -> Contour<MFEKPointData> {
        let mut output_contour: Contour<MFEKPointData> = Vec::new();
        let mut last_curve: Option<[Vector; 4]> = None;

        let mut first_point = true;
        for curve in &self.segs
        {                       
            let control_points = curve.to_control_points();

            let point_type = if first_point && !self.is_closed() { PointType::Move } else { PointType::Curve };
            let mut new_point = control_points[0].to_point(control_points[1].to_handle(), Handle::Colocated, point_type);

            // if this isn't the first point we need to backtrack and set our output point's b handle
            match last_curve
            {
                Some(lc) => {
                    new_point.b = lc[2].to_handle();
                }
                None => {}
            }

            output_contour.push(new_point);

            last_curve = Some(control_points);
            first_point = false;
        }


        if output_contour.len() > 1 && self.is_closed() {
            let fp = output_contour.first_mut().unwrap();

            // we've got to connect the last point and the first point
             fp.b = Vector::to_handle(last_curve.unwrap()[2]);
        }
    
        return output_contour;
    }
}
