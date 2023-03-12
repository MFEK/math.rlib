use std::vec;

use glifparser::glif::contour_operations::vws::VWSHandle;
use glifparser::glif::contour_operations::ContourOperations;
use glifparser::glif::inner::MFEKContourInner;
use glifparser::glif::point::MFEKPointCommon;
use glifparser::glif::point::hyper::HyperPointType;
use glifparser::outline::FromKurbo;
use glifparser::{PointData, Point, WhichHandle, Handle, PointType, Outline, MFEKPointData};
use glifparser::glif::MFEKContour;
use glifparser::glif::contour::MFEKContourCommon;
use spline::SplineSpec;

use crate::{Piecewise, ArcLengthParameterization, Parameterization};

// This method takes a non-cubic contour and resolves it into cubic beziers.
// The implementation should also properly handle resolving the ContourOperation
// such that it 
pub trait ResolveCubic<PD: PointData> {
    fn resolve(&self) -> (Vec<usize>, MFEKContour<PD>);
    fn to_cubic(&self) -> MFEKContour<PD>;

    // Mao an index from the original contour to the resolved cubic
    fn get_index_map(&self) -> Vec<usize>;
}

impl<PD: PointData> ResolveCubic<PD> for MFEKContour<PD> {
    fn to_cubic(&self) -> MFEKContour<PD> {
        return self.resolve().1;
    }

    fn get_index_map(&self) -> Vec<usize> {
        return self.resolve().0
    }

    fn resolve(&self) -> (Vec<usize>, MFEKContour<PD>) {
        match self.inner() {
            MFEKContourInner::Cubic(_) => {
                let index_map = (0..self.len()).collect();
                (index_map, self.clone())
            },
            MFEKContourInner::Quad(contour) => {
                // TODO: Refactor these into functions/files.
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

                let index_map = (0..self.len()).collect();
                (index_map, MFEKContour::new(MFEKContourInner::Cubic(output), self.operation().clone()))
            },
            MFEKContourInner::Hyper(contour) => {
                let contour_len = contour.len();

                let mut spline_spec = SplineSpec::new();
                
                let start_point = tuple_to_kurbo(contour.get_points()[0].get_position());
                spline_spec.move_to(start_point);

                for i in 0..contour_len {
                    let point = &contour.get_points()[i];

                    if let Some(next_point) = contour.next_point(i) {
                        match next_point.kind {
                            HyperPointType::Curve => {
                                let p2 = handle_to_kurbo(point.get_handle(WhichHandle::A));
                                let p3 = handle_to_kurbo(next_point.get_handle(WhichHandle::B));

                                let p4 = tuple_to_kurbo(next_point.get_position());

                                spline_spec.spline_to(p2, p3, p4, next_point.smooth)
                            },
                            HyperPointType::Line => {
                                let p = tuple_to_kurbo(next_point.get_position());
                                spline_spec.line_to(p, next_point.smooth)
                            },
                        }
                    }
                }

                if self.is_closed() { spline_spec.close() }
                let spline = spline_spec.solve();
                
                let mut index_map: Vec<usize> = vec![];
                let mut contour_operation = self.operation().clone();

                if let Some(ContourOperations::VariableWidthStroke{ ref mut data }) = contour_operation {
                    let new_data = data;
                    if let Some(ContourOperations::VariableWidthStroke{ data: _ }) = self.operation() {
                        // NOTE/TODO:
                        // Any op that needs additional handling should be added here.
                        // Finding a way to do this generically across operations would be best!
                        new_data.handles = vec![];
                    }
                }


                let mut final_path = kurbo::BezPath::new();

                if !spline.segments().is_empty() {
                    final_path.move_to(kurbo::Point::new(spline.segments()[0].p0.x, spline.segments()[0].p0.y));

                    let mut last_handle = None;
                    println!("{:?}", spline.segments().iter().count());
                    for (idx, seg) in spline.segments().iter().enumerate() {
                        let mut bez_path = kurbo::BezPath::new();
                        seg.render(&mut bez_path);

                        let mut valid_path = kurbo::BezPath::new();
                        valid_path.move_to(kurbo::Point::new(seg.p0.x, seg.p0.y));
                        valid_path.extend(&bez_path);

                        if let Some(ContourOperations::VariableWidthStroke{ ref mut data }) = contour_operation {
                            let new_data = data;
                            if let Some(ContourOperations::VariableWidthStroke{ ref data }) = self.operation() {
                                let outline: Outline<MFEKPointData> = Outline::from_kurbo(&valid_path);

                                if let Some(pw) = Piecewise::from(&outline).segs.get(0) {
                                    if pw.segs.len() < 1 {
                                        continue;
                                    }
                                    let arc_len = ArcLengthParameterization::from(pw, 100);
                                    
                                    for cut in pw.cuts.iter() {
                                        // TODO: Get VWS interpolation working by turning this bezpath into a piecewise.
                                        let t = arc_len.parameterize(*cut);

                                        let left = (1. - t) * data.handles[idx].left_offset + t * data.handles[idx+1].left_offset;
                                        let right = (1. - t) * data.handles[idx].right_offset + t * data.handles[idx+1].right_offset;
                                        let tangent = (1. - t) * data.handles[idx].tangent_offset + t * data.handles[idx+1].tangent_offset;

                                        let new_handle = VWSHandle {
                                            left_offset: left,
                                            right_offset: right,
                                            tangent_offset: tangent,
                                            interpolation: data.handles[idx].interpolation,
                                        };

                                        last_handle = Some(new_handle);
                                        if *cut == 1. { continue; }
                                        new_data.handles.push(new_handle)
                                    }
                                }
                            }
                        }

                        index_map.push(final_path.segments().count());
                        final_path.extend(bez_path);
                    }

                    if let Some(ContourOperations::VariableWidthStroke{ ref mut data }) = contour_operation {
                        let new_data = data;
                        if let Some(h) = last_handle {
                            new_data.handles.push(h)
                        }
                    }

                    index_map.push(final_path.segments().count());
                }

                if contour.is_closed() {
                    final_path.close_path();
                }

                let outline = Outline::from_kurbo(&final_path);
                let contour = outline.first().unwrap_or(&Vec::new()).clone();
                
                (index_map, MFEKContour::new(MFEKContourInner::Cubic(contour.clone()), contour_operation))
            },
        }
    }
}

fn tuple_to_kurbo(tuple: (f32, f32)) -> kurbo::Point {
    kurbo::Point::new(tuple.0.into(), tuple.1.into())
}

fn handle_to_kurbo(handle: Option<Handle>) -> Option<kurbo::Point> {
    if let Some(Handle::At(x, y)) = handle {
        Some(kurbo::Point::new(x.into(), y.into()))
    } else {
        None
    }
}