mod get_control_points;
use glifparser::{Contour, Handle, Outline, PointType};
pub fn fit(outline: Outline<()>) -> Outline<()> {
    let mut result_outline: Outline<()> = Vec::new();
    for contour in outline.iter() {
        let mut final_contour: Contour<()> = Vec::new();
        let mut last_ptype = PointType::Undefined;
        let mut curve_contour: Contour<()> = Vec::new();
        for point in contour.iter() {
            if point.ptype == PointType::Curve {
                if last_ptype == PointType::Curve {
                    curve_contour.push(point.clone());
                } else {
                    curve_contour.clear();
                    curve_contour.push(point.clone());
                }
            } else {
                if last_ptype == PointType::Curve && !curve_contour.is_empty() {
                    curve_contour.push(point.clone());
                    final_contour.append(&mut solve(curve_contour.clone()));
                    curve_contour.clear();
                }
                final_contour.push(point.clone());
                curve_contour.push(point.clone());
            }
            last_ptype = point.ptype;
        }
        if final_contour.is_empty() {
            final_contour = solve(contour.clone());
        } else if final_contour.len() != contour.len() {
            final_contour.append(&mut solve(curve_contour.clone()));
        }
        result_outline.push(final_contour);
    }
    result_outline
}

fn solve(contour: Vec<glifparser::Point<()>>) -> Vec<glifparser::Point<()>> {
    if contour.len() == 1 {
        return contour;
    }
    let result = get_control_points::get_curve_control_point(contour.clone());
    let mut contour = contour;
    contour[0].a = Handle::At(result.0[0].0, result.0[0].1);
    for i in 0..contour.len() - 1 {
        contour[i].a = {
            if let Some(a) = result.0.get(i) {
                Handle::At(a.0, a.1)
            } else {
                Handle::Colocated
            }
        };
        contour[i + 1].b = {
            if let Some(a) = result.1.get(i) {
                Handle::At(a.0, a.1)
            } else {
                Handle::Colocated
            }
        };
    }
    contour
}
