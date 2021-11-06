use super::Vector;
use glifparser::{Handle, WhichHandle, Point as GPPoint, PointData as GPPointData, PointType as GPPointType};

impl<PD: GPPointData> From<&GPPoint<PD>> for Vector {
    fn from(p: &GPPoint<PD>) -> Self {
        Self::from_point(p)
    }
}

impl<PD: GPPointData> From<GPPoint<PD>> for Vector {
    fn from(p: GPPoint<PD>) -> Self {
        Self::from_point(&p)
    }
}

impl Vector {
    pub fn from_point<PD: GPPointData>(point: &GPPoint<PD>) -> Self
    {
        Vector { x: point.x as f64, y: point.y as f64 }
    }

    pub fn to_point<PD: GPPointData>(self, handle_a: Handle, handle_b: Handle, ptype: GPPointType) -> GPPoint<PD>
    {
        GPPoint::from_x_y_a_b_type((self.x as f32, self.y as f32), (handle_a, handle_b), ptype)
    }

    pub fn from_handle<PD: GPPointData>(point: &GPPoint<PD>, which: WhichHandle) -> Vector
    {
        let handle = match which {
            WhichHandle::A => point.a,
            WhichHandle::B => point.b,
            WhichHandle::Neither => Handle::Colocated,
        };

        match handle {
            Handle::At(x, y) => Vector {x: x as f64, y: y as f64},
            Handle::Colocated => Self::from_point(point),
        }
    }

    pub fn to_handle(self) -> Handle
    {
        Handle::At(self.x as f32, self.y as f32)
    }

}
