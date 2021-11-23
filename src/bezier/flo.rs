use super::super::Vector;
use crate::Piecewise;

use flo_curves::bezier::{BezierCurve, BezierCurveFactory, path::{BezierPath, BezierPathFactory}};
use flo_curves::geo::Geo;


use super::Bezier;

impl Geo for Bezier {
    type Point = Vector;
}

impl BezierCurveFactory for Bezier {
    fn from_points(start: Vector, (control_point1, control_point2): (Vector, Vector), end: Vector) -> Self {
        let bez = Bezier::from_points(start, control_point1, control_point2, end);
        return bez;
    }
}

impl BezierCurve for Bezier {
    fn start_point(&self) -> Self::Point
    {
        self.to_control_points()[0]
    }

    fn end_point(&self) -> Self::Point
    {
        self.to_control_points()[3]
    }

    fn control_points(&self) -> (Self::Point, Self::Point)
    {
        let cp = self.to_control_points();
        (cp[1], cp[2])
    }
}

impl Geo for Piecewise<Bezier> {
    type Point = Vector;
}

impl BezierPath for Piecewise<Bezier> {
    type PointIter = std::vec::IntoIter<(Self::Point, Self::Point, Self::Point)>;

    fn start_point(&self) -> Self::Point {
        self.segs[0].to_control_points()[0]
    }

    fn points(&self) -> Self::PointIter {
        self.segs.iter().map(|s|(s.to_control_points()[1], s.to_control_points()[2], s.to_control_points()[3])).collect::<Vec<_>>().into_iter()
    }
}

impl BezierPathFactory for Piecewise<Bezier> {
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(mut start_point: Self::Point, points: FromIter) -> Self {
        let mut vb: Vec<Bezier> = vec![];
        for p in points.into_iter() {
            vb.push(Bezier::from_points(start_point, p.0, p.1, p.2));
            start_point = p.2;
        }
        Piecewise::new(vb, None)
    }
}
