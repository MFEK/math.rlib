use MFEKmath::Vector;

#[test]
fn conv() {
    let mut v = Vector::from_components(0.0, 100.0);
    v[1] = 50.0;
    v *= 10.0;
    v[0] = v[0] - v[1];
    assert_eq!(v.x, -500.0);
    assert_eq!(v.y, 500.0);
}
