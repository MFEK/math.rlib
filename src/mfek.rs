use glifparser::glif::inner::cubic::MFEKCubicInner;
use glifparser::glif::point::MFEKPointCommon;
use glifparser::{PointData, Point, WhichHandle, Handle, point, PointType};
use glifparser::glif::MFEKContour;
use glifparser::glif::contour::MFEKCubicContour;
use glifparser::glif::point::quad::QPoint;

// This method takes a non-cubic contour and resolves it into cubic beziers.
// The implementation should also properly handle resolving the ContourOperation
// such that it 
pub trait ResolveCubic<PD: PointData> {
    fn resolve_to_cubic(&self) -> MFEKContour<PD>;
}

impl<PD: PointData> ResolveCubic<PD> for MFEKContour<PD> {
    fn resolve_to_cubic(&self) -> MFEKContour<PD> {
        match &self.inner {
            glifparser::glif::inner::MFEKContourInner::Cubic(_) => return self.clone(),
            glifparser::glif::inner::MFEKContourInner::Quad(contour) => {
                let mut output: Vec<Point<PD>> = Vec::new();

                for (idx, point) in contour.iter().enumerate() {
                    let mut cpoint: Point<PD> = Point::new();
                    cpoint.ptype = point.ptype;
                    cpoint.x = point.x;
                    cpoint.y = point.y;

                    if let Handle::At(x, y) = point.a {
                        let hx = point.x + 2./3. * (x - point.x);
                        let hy = point.y + 2./3. * (y - point.y);
                        cpoint.a = Handle::At(hx, hy);
                    }

                    let prev_point_idx = if idx == 0 && contour.len() > 1 && contour[0].ptype != PointType::Move { contour.len() - 1 } else { usize::MAX };
                    if let Some(prev_point) = contour.get(prev_point_idx) {
                        if let Handle::At(x, y) = prev_point.a {
                            let hx = point.x + 2./3. * (x - point.x);
                            let hy = point.y + 2./3. * (y - point.y); 
                            cpoint.b = Handle::At(hx, hy);
                        }
                    }

                    output.push(cpoint);
                }

                let output_outer = MFEKContour {
                    inner: glifparser::glif::inner::MFEKContourInner::Cubic(output),
                    operation: self.operation.clone(),
                };

                return output_outer
            },
        }
    }
}