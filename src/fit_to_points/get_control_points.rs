use glifparser::Contour;
// https://www.codeproject.com/Articles/31859/Draw-a-Smooth-Curve-through-a-Set-of-2D-Points-wit

pub fn get_curve_control_point(knots: Contour<()>) -> (Vec<(f32, f32)>, Vec<(f32, f32)>) {
    let n = knots.len() - 1;

    let mut first_control_point: Vec<(f32, f32)> = Vec::new();
    let mut second_control_point: Vec<(f32, f32)> = Vec::new();
    if n == 1 {
        // Special case: Bezier curve should be a straight line.
        // 3P1 = 2P0 + P3

        first_control_point.push((
            (2. * knots[0].x + knots[1].x) / 3.,
            (2. * knots[0].y + knots[1].y) / 3.,
        ));
        // P2 = 2P1 – P0
        second_control_point.push((
            2. * first_control_point[0].0 - knots[0].x,
            2. * first_control_point[0].1 - knots[0].y,
        ));
    } else {
        // Calculate first Bezier control points
        // Right hand side vector
        // Set right hand side X values
        let mut rhs = vec![knots[0].x + 2. * knots[1].x];
        for i in 1..(n - 1) {
            rhs.push(4. * knots[i].x + 2. * knots[i + 1].x);
        }
        rhs.push((8. * knots[n - 1].x + knots[n].x) / 2.0);
        // Get first control points X-values
        let x = get_first_control_points(rhs.clone());
        // Set right hand side Y values
        for i in 1..(n - 1) {
            rhs[i] = 4. * knots[i].y + 2. * knots[i + 1].y;
        }
        rhs[0] = knots[0].y + 2. * knots[1].y;
        rhs[n - 1] = (8. * knots[n - 1].y + knots[n].y) / 2.0;
        // Get first control points Y-values
        let y = get_first_control_points(rhs);
        // Fill output arrays
        for i in 0..n {
            first_control_point.push((x[i], y[i]));
            if i < n - 1 {
                second_control_point.push((
                    2. * knots[i + 1].x - x[i + 1],
                    2. * knots[i + 1].y - y[i + 1],
                ));
            } else {
                second_control_point
                    .push(((knots[n].x + x[n - 1]) / 2., (knots[n].y + y[n - 1]) / 2.))
            }
        }
    }
    (first_control_point, second_control_point)
}

fn get_first_control_points(rhs: Vec<f32>) -> Vec<f32> {
    let n = rhs.len();
    let mut x = Vec::<f32>::new(); //solution vector
    let mut tmp = Vec::<f32>::new();
    let mut b = 2.;
    x.push(rhs[0] / b);
    tmp.push(0.);
    for i in 1..n {
        //decomposition and forward substitution
        tmp.push(1. / b);
        b = (if i < n - 1 { 4.0 } else { 3.5 }) - tmp[i];
        x.push((rhs[i] - x[i - 1]) / b);
    }
    for i in 1..n {
        x[n - i - 1] -= tmp[n - i] * x[n - i]; // Backsubstitution.
    }
    x
}
