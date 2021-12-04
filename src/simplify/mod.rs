use glifparser::{Contour, Outline, Point};
pub mod get_control_points;
pub fn simplify(point: Outline<()>) -> Outline<()> {
    let mut points = point.clone();
    drop(point);
    let mut result: Outline<()> = Vec::new();
    for i in points.iter() {
        
        let result_a = get_control_points::get_curve_control_point(solution(i.clone())).unwrap();

        result.push(result_a.0);
        result.push(result_a.1);
    }
    points
}

fn solution(points: Contour<()>) -> Contour<()> {
    let mut points = points.clone();
    let mut n = points.len();
    let mut starti = 0;
    while starti < n - 2 {
        let mut i = starti + 1;
        while i < n - 2 {
            let P1 = points[starti].clone();
            let P2 = points[i].clone();
            let P3 = points[i + 1].clone();
            let S1 = slope(P1.clone(), P2);
            let S2 = slope(P1, P3);
            if S1 == S2 {
                points.remove(i);
            } else {
                break;
            }
            i += 1;
            n = points.len();
        }
        starti += 1;
        dbg!(&n);
    }
    points
}
fn slope(P1: Point<()>, P2: Point<()>) -> f32 {
    (P2.y - P1.y) / (P2.x - P1.x)
}
