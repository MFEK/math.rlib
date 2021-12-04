use glifparser::Point;
use MFEKmath::simplify;

#[test]
fn test_() {
    let points: Vec<Point<()>> = vec![
        Point::from_x_y_type((0., 0.), glifparser::PointType::Undefined),
        // Point::from_x_y_type((0., 0.), glifparser::PointType::Undefined),
        // Point::from_x_y_type((0.114, 1.005), glifparser::PointType::Undefined),
        Point::from_x_y_type((0.5, 1.518), glifparser::PointType::Undefined),
        // Point::from_x_y_type((0.905, 0.936), glifparser::PointType::Undefined),
        Point::from_x_y_type((1., 0.), glifparser::PointType::Undefined),
    ];
    let (first, second) =
        simplify::get_control_points::get_curve_control_point(points).expect("Failed");
    dbg!(&first);
    dbg!(second);
    // assert_eq!(first[0].x, 0.);
    // assert_eq!(first[0].y, 1.);
}
