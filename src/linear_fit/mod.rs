use crate::Vector;

pub fn linear_fit(points: Vec<Vector>) -> (f64, f64) {
    let a: f64;
    let b: f64;
    let (mut xsum, mut x2sum, mut ysum, mut xysum) = (0., 0., 0., 0.);
    for i in 0..points.len() {
        xsum = xsum + points[i].x; //sigma x
        ysum = ysum + points[i].y; //sigma y
        x2sum = x2sum + (points[i].x * points[i].x); //sigma x^s
        xysum = xysum + points[i].x * points[i].y; //sigm xy
    }
    let n = points.len() as f64;
    a = (n * xysum - xsum * ysum) / (n * x2sum - xsum * xsum); //calculate slope
    b = (x2sum * ysum - xsum * xysum) / (x2sum * n - xsum * xsum); //calculate intercept
    let mut y_fit = Vec::<f64>::new();
    for i in 0..n as usize {
        y_fit.push(a * points[i].x + b) //to calculate y(fitted) at given x points (y=ma+b) m is slope and c in intercept
    }
    (a, b)
}
