use MFEKmath::Vector;

fn find_dist(left: Vector, right: Vector, point: Vector) -> f64 {
    f64::abs((point.y - left.y) * (right.x - left.x) - (right.y - left.y) * (point.x - left.x))
}
fn find_side(left: Vector, right: Vector, point: Vector) -> i32 {
    let val = (point.y - left.y) * (right.x - left.x) - (right.y - left.y) * (point.x - left.x);
    if val > 0. {
        1
    } else {
        -1
    }
}
fn quick_hull(
    points: &Vec<Vector>,
    left: Vector,
    right: Vector,
    side: i32,
    hull: &mut Vec<Vector>,
) {
    let mut ind = -1; //     int ind = -1;
    let mut max_dist = 0.; // int max_dist = 0;

    for i in 0..points.len() {
        let dist = find_dist(left, right, points[i].clone());
        if find_side(left, right, points[i].clone()) == side && dist > max_dist {
            ind = i as i32;
            max_dist = dist;
        }
    }

    if ind == -1 {
        hull.push(left);
        hull.push(right);
        return;
    }
    quick_hull(
        points,
        points[ind as usize],
        left,
        -find_side(points[ind as usize], left, right),
        hull,
    );
    quick_hull(
        points,
        points[ind as usize],
        right,
        -find_side(points[ind as usize], right, left),
        hull,
    );
}
#[allow(non_snake_case, unused)]
pub fn quickHull<T>(points: Vec<Vector>) {
    let mut hull = Vec::<Vector>::new();
    let mut maxx = 0.;
    let mut minx = 0.;
    let mut maxi: usize = 0;
    let mut mini: usize = 0;

    for (i, point) in points.iter().enumerate() {
        if point.x > maxx {
            maxx = point.x;
            maxi = i;
        } else if point.x < minx {
            minx = point.x;
            mini = i;
        }
    }
    let left = points[mini];
    let right = points[maxi];
    quick_hull(&points, left, right, 1, &mut hull);
    quick_hull(&points, left, right, -1, &mut hull)
}
