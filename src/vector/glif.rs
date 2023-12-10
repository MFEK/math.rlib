use super::Vector;
use glifparser::{Handle, WhichHandle, Point as GPPoint, PointData as GPPointData, PointType as GPPointType, glif::point::quad::QPoint};

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

    pub fn from_quad_point<PD: GPPointData>(point: &QPoint<PD>) -> Self {
        Self::from_components(point.x as f64, point.y as f64)
    }

    pub fn from_quad_handle<PD: GPPointData>(point: &QPoint<PD>) -> Self {
        match point.a {
            Handle::At(x, y) => {
                Self::from_components(x as f64, y as f64)
            },
            Handle::Colocated => return Self::from_quad_point(point),
        }    
    }

    pub fn to_point<PD: GPPointData>(self, handle_a: Handle, handle_b: Handle, ptype: GPPointType) -> GPPoint<PD>
    {
        GPPoint::from_x_y_a_b_type((self.x as f32, self.y as f32), (handle_a, handle_b), ptype)
    }

    pub fn to_quad_point<PD: GPPointData>(self, handle_a: Handle) -> QPoint<PD> {
        QPoint { x: self.x as f32, y: self.y as f32, a: handle_a, ptype: GPPointType::Curve, ..Default::default()}
    }

    pub fn from_handle<PD: GPPointData>(point: &GPPoint<PD>, handle: Handle) -> Self {
        match handle {
            Handle::At(x, y) => {
                Self::from_components(x as f64, y as f64)
            },
            Handle::Colocated => return Self::from_point(point),
        }
    }

    pub fn from_handle_enum<PD: GPPointData>(point: &GPPoint<PD>, which: WhichHandle) -> Vector
    {
        let handle = match which {
            WhichHandle::A => point.a,
            WhichHandle::B => point.b,
            WhichHandle::Neither => Handle::Colocated,
        };

        Self::from_handle(point, handle)
    }

    pub fn to_handle(self) -> Handle
    {
        Handle::At(self.x as f32, self.y as f32)
    }

}
