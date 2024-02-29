use glifparser::glif::mfek::point::MFEKPointCommon;
/// This module adds math functions to types in glifparser.
use glifparser::{Handle, Point, PointData, PointType, WhichHandle};

use std::f32::consts;

use crate::bezier::Bezier;
use crate::vector::Vector;

pub trait PolarCoordinates {
    /// Considering the point location as the origin, returns handle's position in Cartesian
    /// coordinates (irrespective of glyph origin)
    fn cartesian(&self, wh: WhichHandle) -> (f32, f32);
    /// Returns theta (ϑ) in radians (_Cf._ [`f32::to_radians`])
    fn polar(&self, wh: WhichHandle) -> (f32, f32);
    /// Expects theta (ϑ) in degrees (_Cf._ [`f32::to_degrees`])
    fn set_polar(&mut self, wh: WhichHandle, polar: (f32, f32));
}

impl Bezier {
    fn point_from_bezier_handle(&self, wh: WhichHandle) -> Point<()> {
        let (p, h) = match wh {
            WhichHandle::A => (self.w1, self.w2),
            WhichHandle::B => (self.w4, self.w3),
            WhichHandle::Neither => unreachable!(),
        };
        let mut gp: Point<()> =
            Vector::to_point(p, Handle::Colocated, Handle::Colocated, PointType::Line);
        let gh: Handle = Vector::to_handle(h);
        match wh {
            WhichHandle::A => {
                gp.a = gh;
            }
            WhichHandle::B => {
                gp.b = gh;
            }
            WhichHandle::Neither => unreachable!(),
        }
        gp
    }
}

impl PolarCoordinates for Bezier {
    fn cartesian(&self, wh: WhichHandle) -> (f32, f32) {
        let p = Bezier::point_from_bezier_handle(self, wh);
        p.cartesian(wh)
    }
    fn polar(&self, wh: WhichHandle) -> (f32, f32) {
        let p = Bezier::point_from_bezier_handle(self, wh);
        p.polar(wh)
    }
    fn set_polar(&mut self, wh: WhichHandle, polar: (f32, f32)) {
        let mut p = Bezier::point_from_bezier_handle(self, wh);
        p.set_polar(WhichHandle::Neither, polar);
        let h = &mut match wh {
            WhichHandle::A => self.w2,
            WhichHandle::B => self.w3,
            WhichHandle::Neither => unreachable!(),
        };
        h.x = p.x as f64;
        h.y = p.y as f64;
    }
}

use WhichHandle::{Neither, A, B};
impl<PD: PointData> PolarCoordinates for Point<PD> {
    fn cartesian(&self, wh: WhichHandle) -> (f32, f32) {
        let (x, y) = match wh {
            Neither => (self.x, self.y),
            A => self.handle_or_colocated(WhichHandle::A, &|f| f, &|f| f),
            B => self.handle_or_colocated(WhichHandle::B, &|f| f, &|f| f),
        };
        (self.x - x, self.y - y)
    }
    fn polar(&self, wh: WhichHandle) -> (f32, f32) {
        let (x, y) = self.cartesian(wh);
        let r = (x.powf(2.) + y.powf(2.)).sqrt();
        let theta = y.atan2(x);
        (r, theta)
    }
    fn set_polar(&mut self, wh: WhichHandle, (r, theta): (f32, f32)) {
        let x = self.x + (r * (theta * (consts::PI / 180.)).cos());
        let y = self.y + (r * (theta * (consts::PI / 180.)).sin());

        match wh {
            Neither => {
                self.x = x;
                self.y = y;
            }
            A => {
                self.a = Handle::At(x, y);
            }
            B => {
                self.b = Handle::At(x, y);
            }
        };
    }
}

impl<PD: PointData> PolarCoordinates for &mut dyn MFEKPointCommon<PD> {
    fn cartesian(&self, wh: WhichHandle) -> (f32, f32) {
        let (x, y) = match wh {
            Neither => (self.x(), self.y()),
            A => self.get_handle_position(WhichHandle::A).unwrap(),
            B => self.get_handle_position(WhichHandle::B).unwrap(),
        };
        (self.x() - x, self.y() - y)
    }
    fn polar(&self, wh: WhichHandle) -> (f32, f32) {
        let (x, y) = self.cartesian(wh);
        let r = (x.powf(2.) + y.powf(2.)).sqrt();
        let theta = y.atan2(x);
        (r, theta)
    }
    fn set_polar(&mut self, wh: WhichHandle, (r, theta): (f32, f32)) {
        let x = self.x() + (r * (theta * (consts::PI / 180.)).cos());
        let y = self.y() + (r * (theta * (consts::PI / 180.)).sin());

        match wh {
            Neither => self.set_position(x, y),
            A => self.set_handle(WhichHandle::A, Handle::At(x, y)),
            B => self.set_handle(WhichHandle::B, Handle::At(x, y)),
        };
    }
}
