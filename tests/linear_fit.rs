use MFEKmath::{linear_fit::linear_fit, Vector};

#[test]
fn test_() {
    let points = vec![
        Vector::from_components(10., 10.),
        Vector::from_components(20., 20.),
        Vector::from_components(100., 100.),
        Vector::from_components(120., 120.),
    ];
    let (a, b) = linear_fit(points);
    assert_eq!(a, 1.0);
    assert_eq!(b, 0.0);

    let points = vec![
        Vector::from_components(2., 1.),
        Vector::from_components(4., 1.),
        Vector::from_components(3., 5.),
        Vector::from_components(5., 6.),
    ];
    let (a, b) = linear_fit(points);
    // assert_eq!(a, 1.0);
    // assert_eq!(b, 0.0);
    println!("{} {}", a, b);
}
