use glifparser::{Outline, PointData};

pub trait Fixup {
    fn assert_colocated(&mut self);
    fn assert_colocated_within(&mut self, within: f32);
}

impl<PD: PointData> Fixup for Outline<PD> {
    fn assert_colocated(&mut self) {
        self.assert_colocated_within(1.);
    }

    fn assert_colocated_within(&mut self, within: f32) {
        for c in self.iter_mut() {
            for p in c.iter_mut() {
                if let glifparser::Handle::At(ax, ay) = p.a {
                    if (ax - p.x).abs() < within && (ay - p.y).abs() < within { p.a = glifparser::Handle::Colocated; }
                }
                if let glifparser::Handle::At(bx, by) = p.b {
                    if (bx - p.x).abs() < within && (by - p.y).abs() < within { p.b = glifparser::Handle::Colocated; }
                }
            }
        }   
    }
}
