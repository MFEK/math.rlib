use glifparser::Point;

pub fn simplify(point: Vec<Vec<Point<()>>>) {
    let mut points = point.clone();
    drop(point);
    for i in points.iter_mut() {
        solution(i);
    }
}
fn solution(points: &mut Vec<Point<()>>) {
    let n = points.len();
    for starti in 0..n {
        for i in (starti + 1)..n {
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
        }
    }
}
fn slope(P1: Point<()>, P2: Point<()>) -> f32 {
    (P2.y - P1.y) / (P2.x - P1.x)
}
