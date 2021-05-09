/// This module adds math functions to types in glifparser.

use glifparser::{Handle, Point, WhichHandle};
use glifparser::PointData;

use std::f32::consts;

pub trait PolarCoordinates {
    fn cartesian(&self, wh: WhichHandle) -> (f32, f32);
    fn polar(&self, wh: WhichHandle) -> (f32, f32);
    fn set_polar(&mut self, wh: WhichHandle, polar: (f32, f32));
}

impl<PD: PointData> PolarCoordinates for Point<PD> {
    fn cartesian(&self, wh: WhichHandle) -> (f32, f32) {
        use WhichHandle::*;
        let (x, y) = match wh {
            Neither => (self.x, self.y),
            A => self.handle_or_colocated(WhichHandle::A, |f|f, |f|f),
            B => self.handle_or_colocated(WhichHandle::B, |f|f, |f|f),
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

        use WhichHandle::*;
        match wh {
            Neither => {self.x = x; self.y = y;},
            A => {self.a = Handle::At(x, y);},
            B => {self.b = Handle::At(x, y);},
        };
    }
}



