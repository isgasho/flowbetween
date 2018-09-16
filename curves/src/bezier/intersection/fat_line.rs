use super::super::curve::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

use std::f64;

///
/// A 'fat line' is a line with a width. It's used in bezier intersection algorithms,
/// in particular the clipping algorithm described by Sederberg and Nishita
/// 
pub struct FatLine<Line> {
    /// The thin line L'
    line: Line,

    /// The distance from the line to the upper part of the 'fat line'
    d_min: f64,

    /// The distance from the line to the lower part of the 'fat line'
    d_max: f64,

    /// The coefficients (a, b, c) in the equation ax+bx+c (where a^2+b^2 = 0)
    coeff: (f64, f64, f64)
}

impl<L: Line> FatLine<L>
where L::Point: Coordinate2D {
    ///
    /// Creates a new fat line
    /// 
    pub fn new(line: L, d_min: f64, d_max: f64) -> FatLine<L> {
        let (from, to)  = line.points();
        let (a, b, c)   = line_coefficients_2d(&line);

        FatLine {
            line:   line,
            d_min:  d_min,
            d_max:  d_max,
            coeff:  (a, b, c)
        }
    }

    ///
    /// Returns the distance between the point and the central line
    /// 
    #[inline]
    pub fn distance(&self, point: &L::Point) -> f64 {
        let (a, b, c) = self.coeff;
        a*point.x() + b*point.y() + c
    }

    ///
    /// Given a bezier curve, returns another curve whose X axis is the distance
    /// from the central line and the Y axis varies from 0 to 1, with a uniform
    /// distribution of t values.
    /// 
    /// This is used in the bezier clipping algorithm to discover where a bezier
    /// curve clips against this line.
    /// 
    pub fn distance_curve<C: BezierCurveFactory<Point=L::Point>>(&self, curve: &C) -> C {
        let (cp1, cp2)  = curve.control_points();

        let start       = L::Point::from_components(&[self.distance(&curve.start_point()), 0.0]);
        let end         = L::Point::from_components(&[self.distance(&curve.end_point()), 1.0]);
        let cp1         = L::Point::from_components(&[self.distance(&cp1), 1.0/3.0]);
        let cp2         = L::Point::from_components(&[self.distance(&cp2), 2.0/3.0]);

        C::from_points(start, end, cp1, cp2)
    }

    ///
    /// Returns the convex hull of a curve returned by distance_curve
    /// 
    /// We can use some of the properties of the distance_curve to simplify how this
    /// is worked out (specifically, we know the points are sorted vertically already
    /// so we only need to know if the two control points are on the same side or not)
    /// 
    fn distance_curve_convex_hull<C: BezierCurve<Point=L::Point>>(distance_curve: &C) -> Vec<L::Point> {
        // Read the points from the curve
        let start       = distance_curve.start_point();
        let (cp1, cp2)  = distance_curve.control_points();
        let end         = distance_curve.end_point();

        // Compute the x component of the distances of cp1 and cp2 from the central line defined by start->end
        // These are the m and c values for y=mx+c assuming that start.y() = 0 and end.y() = 1 which is true for the distance curve
        let m = end.x()-start.x();
        let c = start.x();

        let dx1 = cp1.x() - (m*(1.0/3.0)+c);
        let dx2 = cp2.x() - (m*(2.0/3.0)+c);

        // If they have the same sign, they're on the same side
        let on_same_side = dx1*dx2 >= 0.0;

        // Ordering on the convex hull depends only on if cp1 and cp2 are on the same side or not
        if on_same_side {
            // cp1 or cp2 might be inside the hull
            let dist_ratio = dx1/dx2;

            if dist_ratio >= 2.0 {
                // cp2 is in the hull (between the line cp1->end and start->end)
                vec![start, cp1, end]
            } else if dist_ratio <= 0.5 {
                // cp1 is in the hull (between the line cp2->end and start->end)
                vec![start, cp2, end]
            } else {
                // All points are on the hull
                vec![start, cp1, cp2, end]
            }
        } else {
            // It's not possible to have a point inside the hull
            vec![start, cp1, end, cp2]
        }
    }

    ///
    /// Rounds values very close to 0 or 1 to 0 or 1
    /// 
    #[inline]
    fn round_y_value(y: f64) -> f64 {
        if y < 0.0 && y > -0.001 {
            0.0
        } else if y > 1.0 && y < 1.001 {
            1.0
        } else {
            y
        }
    }

    ///
    /// Given an x pos on a line, solves for the y point
    /// 
    #[inline]
    fn solve_line_y((x1, x2): (f64, f64), (p1, p2): (&L::Point, &L::Point)) -> (Option<f64>, Option<f64>) {
        let min_x = p1.x().min(p2.x());
        let max_x = p1.x().max(p2.x());

        let m = (p2.y()-p1.y())/(p2.x()-p1.x());
        let c = p1.y() - m * p1.x();

        let y1 = if x1 >= min_x && x1 <= max_x { Some(Self::round_y_value(m*x1 + c)) } else { None };
        let y2 = if x2 >= min_x && x2 <= max_x { Some(Self::round_y_value(m*x2 + c)) } else { None };

        (y1, y2)
    }

    ///
    /// Finds the t values where a bezier curve clips against this fat line, or returns
    /// None if there are no t values on the specified curve that are inside the line
    /// 
    pub fn clip_t<C: BezierCurveFactory<Point=L::Point>>(&self, curve: &C) -> Option<(f64, f64)> {
        // The 'distance' curve is a bezier curve where 'x' is the distance to the central line from the curve and 'y' is the t value where that distance occurs
        let distance_curve          = self.distance_curve(curve);

        // The convex hull encloses the distance curve, and can be used to find the y values where it's between d_min and d_max
        // As y=t due to how we construct the distance curve these are also the t values
        // We make use of the fact that the hull always has the start point at the start
        let distance_convex_hull    = Self::distance_curve_convex_hull(&distance_curve);

        // To solve for t, we need to find where the two edge lines cross d_min and d_max
        let num_points  = distance_convex_hull.len();
        let mut t1 = f64::MAX;
        let mut t2 = f64::MIN;
        for idx in 0..num_points {
            // Solve where this part of the convex hull crosses this line
            let l           = (&distance_convex_hull[idx], &distance_convex_hull[(idx+1)%num_points]);
            let (t1a, t2a)  = Self::solve_line_y((self.d_min, self.d_max), l);

            if let Some(t1a) = t1a {
                if t1a > 0.0 && t1a < 1.0 { t1 = t1.min(t1a) }
            }
            if let Some(t2a) = t2a {
                if t2a > 0.0 && t2a < 1.0 { t2 = t2.max(t2a) }
            }
        }

        if t1 > t2 {
            // No part of the hull crossed the line
            None
        } else if t1 < 0.0 {
            // t2 may still be > 0.0 and form a valid line
            if t2 < 0.0 {
                None
            } else if t2 > 1.0 {
                Some((0.0, 1.0))
            } else {
                Some((0.0, t2))
            }
        } else if t1 > 1.0 {
            // t2 must be larger than t1 so no clip
            None
        } else {
            // Both in the range 0-1
            Some((t1, t2))
        }
    }

    ///
    /// Clips a bezier curve against this fat line
    /// 
    pub fn clip<C: BezierCurveFactory<Point=L::Point>>(&self, curve: &C) -> Option<C> {
        if let Some((t1, t2)) = self.clip_t(curve) {
            Some(curve.subdivide::<C>(t1).1.subdivide((t2-t1)/(1.0-t1)).0)
        } else {
            None
        }
    }
}

impl<P: Coordinate+Coordinate2D> FatLine<(P, P)> {
    ///
    /// Creates a new fatline from a curve
    /// 
    pub fn from_curve<C: BezierCurve<Point=P>>(curve: &C) -> FatLine<(P, P)> {
        // Line between the start and end points of the curve
        let line        = (curve.start_point(), curve.end_point());
        
        // Coefficients for the line
        let (a, b, c)   = line_coefficients_2d(&line);

        // Compute the distances to the control points
        let (cp1, cp2)  = curve.control_points();
        let d1          = a*cp1.x() + b*cp1.y() + c;
        let d2          = a*cp2.x() + b*cp2.y() + c;

        // This is the 'estimated fit' shortcut suggested by Sederberg/Nishta in their paper rather than the tighest ffitting line
        let (d_min, d_max) = if d1*d2 > 0.0 {
            // Both control points on the same side of the line
            (
                (3.0/4.0) * (d1.min(d2).min(0.0)),
                (3.0/4.0) * (d1.max(d2).max(0.0))
            )
        } else {
            // Control points on opposite sides of the line
            (
                (4.0/9.0) * (d1.min(d2).min(0.0)),
                (4.0/9.0) * (d1.max(d2).max(0.0))
            )
        };

        FatLine {
            line:   line,
            d_min:  d_min,
            d_max:  d_max,
            coeff:  (a, b, c)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::super::*;
    use super::super::super::super::bezier::*;

    #[test]
    fn distance_to_horizontal_line() {
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);

        assert!((fat_line.distance(&Coord2(0.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(0.0, 0.0))- -4.0).abs() < 0.0001);

        assert!((fat_line.distance(&Coord2(3.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(3.0, 0.0))- -4.0).abs() < 0.0001);

        assert!((fat_line.distance(&Coord2(5.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(5.0, 0.0))- -4.0).abs() < 0.0001);

        assert!((fat_line.distance(&Coord2(200.0, 8.0))-4.0).abs() < 0.0001);
        assert!((fat_line.distance(&Coord2(200.0, 0.0))- -4.0).abs() < 0.0001);
    }

    #[test]
    fn convex_hull_basic() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(5.0, 1.0/3.0), Coord2(6.0, 2.0/3.0));
        let hull        = FatLine::<(Coord2, Coord2)>::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==4);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(5.0, 1.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(6.0, 2.0/3.0)) < 0.001);
        assert!(hull[3].distance_to(&Coord2(4.0, 1.0)) < 0.001);
    }

    #[test]
    fn convex_hull_concave_cp2() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(4.0, 1.0/3.0), Coord2(3.0, 2.0/3.0));
        let hull        = FatLine::<(Coord2, Coord2)>::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==3);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(4.0, 1.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(4.0, 1.0)) < 0.001);
    }

    #[test]
    fn convex_hull_concave_cp1() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(4.0, 1.0/3.0), Coord2(8.0, 2.0/3.0));
        let hull        = FatLine::<(Coord2, Coord2)>::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==3);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(8.0, 2.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(4.0, 1.0)) < 0.001);
    }

    #[test]
    fn convex_hull_opposite_sides() {
        let hull_curve  = Curve::from_points(Coord2(1.0, 0.0), Coord2(4.0, 1.0), Coord2(4.0, 1.0/3.0), Coord2(1.0, 2.0/3.0));
        let hull        = FatLine::<(Coord2, Coord2)>::distance_curve_convex_hull(&hull_curve);

        println!("{:?}", hull);

        assert!(hull.len()==4);
        assert!(hull[0].distance_to(&Coord2(1.0, 0.0)) < 0.001);
        assert!(hull[1].distance_to(&Coord2(4.0, 1.0/3.0)) < 0.001);
        assert!(hull[2].distance_to(&Coord2(4.0, 1.0)) < 0.001);
        assert!(hull[3].distance_to(&Coord2(1.0, 2.0/3.0)) < 0.001);
    }

    #[test]
    fn distance_curve_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line        = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve      = line_to_bezier::<_, Curve<_>>(&(Coord2(0.0, 0.0), Coord2(5.0, 8.0)));
        let distance_curve  = fat_line.distance_curve(&clip_curve);

        println!("{:?} {:?}", distance_curve.point_at_pos(0.0), distance_curve.point_at_pos(1.0));

        assert!((distance_curve.point_at_pos(0.0).x()- -4.0).abs() < 0.0001);
        assert!((distance_curve.point_at_pos(1.0).x()-4.0).abs() < 0.0001);
    }

    #[test]
    fn clip_line_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve  = line_to_bezier::<_, Curve<_>>(&(Coord2(0.0, 0.0), Coord2(5.0, 8.0)));

        let clipped     = fat_line.clip(&clip_curve).unwrap();
        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.y()-2.0).abs() < 0.001);
        assert!((end_point.y()-7.0).abs() < 0.001);
    }

    #[test]
    fn clip_t_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line        = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve      = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));
        let distance_curve  = fat_line.distance_curve(&clip_curve);

        let (t1, t2)    = fat_line.clip_t(&clip_curve).unwrap();
        let start_point = clip_curve.point_at_pos(t1);
        let end_point   = clip_curve.point_at_pos(t2);

        println!("Points on curve: {:?} {:?}", start_point, end_point);
        println!("Distance-x: {:?} {:?}", distance_curve.point_at_pos(t1).x(), distance_curve.point_at_pos(t2).x());
        println!("Distance-y: {:?} {:?}", distance_curve.point_at_pos(t1).y(), distance_curve.point_at_pos(t2).y());
        println!("T: {:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.y()-2.0).abs() < 0.0001);
        assert!((end_point.y()-7.0).abs() < 0.0001);
    }

    #[test]
    fn clip_curve_1() {
        // Horizontal line, with a y range of 2.0 to 7.0
        let fat_line    = FatLine::new((Coord2(0.0, 4.0), Coord2(5.0, 4.0)), -2.0, 3.0);
        let clip_curve  = Curve::from_points(Coord2(0.0, 0.0), Coord2(5.0, 8.0), Coord2(0.0, 5.0), Coord2(5.0, 4.0));

        let mut clipped = clip_curve.clone();

        for x in 0..5 {
            let start_point = clipped.point_at_pos(0.0);
            let end_point   = clipped.point_at_pos(1.0);

            let next_clipped = fat_line.clip(&clipped).unwrap();
            clipped = next_clipped;
        }

        let start_point = clipped.point_at_pos(0.0);
        let end_point   = clipped.point_at_pos(1.0);

        println!("{:?} {:?}", start_point, end_point);
        println!("{:?}", fat_line.clip_t(&clip_curve));

        assert!((start_point.y()-2.0).abs() < 0.0001);
        assert!((end_point.y()-7.0).abs() < 0.0001);
    }
}
