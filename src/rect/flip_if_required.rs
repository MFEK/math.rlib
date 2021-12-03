use skia_safe as sk;

/// Skia Contains trait doesn't recognize a point as being contained if the rectangle is drawn
/// backwards or upside-down. This corrects for that.
pub trait FlipIfRequired {
    fn flip_if_required(&mut self);
}

impl FlipIfRequired for sk::Rect {
    fn flip_if_required(&mut self) {
        if self.right < self.left {
            let l = self.left;
            let r = self.right;
            self.right = l;
            self.left = r;
        }

        if self.bottom < self.top {
            let b = self.bottom;
            let t = self.top;
            self.top = b;
            self.bottom = t;
        }
    }
}
