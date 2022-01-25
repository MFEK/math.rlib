use glifparser::{Contour, Handle, Outline, PointType};

fn slope(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    (p2.1 - p1.1) / (p2.0 - p1.0)
}

fn in_line(slope_1: f32, slope_2: f32) -> bool {
    slope_1 == slope_2 || (slope_1 - slope_2).abs() < 0.1
}
fn detect_line(contour: &mut Contour<()>) {
 
    for i in 0..contour.len() - 1 {
        if contour[i].ptype == PointType::Line {
            continue;
        }
        let p1 = contour[i].clone();
        let p2 = contour[i + 1].clone();
        let b = match p2.b {
            Handle::Colocated => (0.0, 0.0),
            Handle::At(x, y) => (x, y),
        };
        let a = match p1.a {
            Handle::Colocated => (0.0, 0.0),
            Handle::At(x, y) => (x, y),
        };
        let p1 = (p1.x, p1.y);
        let p2 = (p2.x, p2.y);
        let m = slope(p1, p2);
        let a_slope = slope(a, p1);
        let a_on_line = in_line(a_slope, m);
        let b_slope = slope(b, p2);
        let b_on_line = in_line(b_slope, m);
     
        if a_on_line && b_on_line {
            if contour[i].ptype != PointType::Move {
                contour[i].ptype = PointType::Line;
            }
            contour[i].a = Handle::Colocated;
            contour[i + 1].ptype = PointType::Line;
            contour[i + 1].b = Handle::Colocated;
        }
    }
    
}
pub fn simplify(outline: Outline<()>) -> Outline<()> {
    // detect_line()
    let mut outline = outline;
    for contour in outline.iter_mut() {
        detect_line(contour);
    }

    outline
}
