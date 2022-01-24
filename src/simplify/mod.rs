use glifparser::{outline, Contour, Handle, Outline, Point, PointType};
fn detect_line_2(contour: &mut Contour<()>) {
    let mut i = 0;
    while i < contour.len() - 2 {
        if contour[i].ptype == PointType::Line
            && contour[i + 1].ptype == PointType::Line
            && contour[i + 2].ptype == PointType::Line
        {
            let p1 = (contour[i].x, contour[i].y);
            let p = (contour[i + 1].x, contour[i].y);
            let p2 = (contour[i + 2].x, contour[i].y);
            if equation(p1, p2, p) {
                contour.remove(i + 1);
                println!("rm {}", i + 1);
            }
            i += 1;
        } else {
            i += 1;
        }
    }
}

fn equation(p1: (f32, f32), p2: (f32, f32), p: (f32, f32)) -> bool {
    let m = slope(p1, p2);
    let p_slope = slope(p, p1);
    m == p_slope
}

fn slope(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    (p2.1 - p1.1) / (p2.0 - p1.0)
}

fn in_line(slope_1: f32, slope_2: f32) -> bool {
    slope_1 == slope_2 || (slope_1 - slope_2).abs() < 0.1
}
fn detect_line(contour: &mut Contour<()>, final_: bool) {
    // dbg!(&contour);
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
        // let a_on_line=equation(p1, p2, a);
        // let b_on_line=equation(p1,p2,b);
        let b_slope = slope(b, p2);
        let b_on_line = in_line(b_slope, m);
        // let a_on_line = equation(p1, p2, a);
        // let b_on_line = equation(p1, p2, b);
        // dbg!(&p1);
        // dbg!(&p2);
        // dbg!(&a);
        // dbg!(&a_on_line);
        // dbg!(&a_slope);
        // dbg!(&b);
        // dbg!(&b_slope);
        // dbg!(&b_on_line);
        // dbg!(&m);
        dbg!(&i);
        if a_on_line && b_on_line {
            if contour[i].ptype != PointType::Move {
                contour[i].ptype = PointType::Line;
            }
            contour[i].a = Handle::Colocated;
            contour[i + 1].ptype = PointType::Line;
            contour[i + 1].b = Handle::Colocated;
        }
    }
    if final_ {
        detect_line_2(contour);
    }
}
pub fn simplify(outline: Outline<()>, final_: bool) -> Outline<()> {
    // detect_line()
    let mut outline = outline;
    for contour in outline.iter_mut() {
        detect_line(contour, final_);
    }
    println!("------------------------------");

    outline
}
