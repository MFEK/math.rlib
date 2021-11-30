use MFEKmath::{linear_fit::linear_fit, Vector};

#[test]
fn test_() {
    let points = vec![
        Vector::from_components(50., 12.),
        Vector::from_components(70., 15.),
        Vector::from_components(100., 21.),
        Vector::from_components(120., 25.),
    ];
    let (a, b) = linear_fit(points);
    assert_eq!(a, 0.187931);
    assert_eq!(b, 2.27586);
}
