use super::*;
use crate::material::Material;
use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub enum RectAxis {
    XY,
    XZ,
    YZ,
}

impl RectAxis {
    fn point(&self, (a, b): (f64, f64), k: f64) -> Point {
        use RectAxis::*;
        match self {
            XY => Point::new(a, b, k),
            XZ => Point::new(a, k, b),
            YZ => Point::new(k, a, b),
        }
    }

    fn k(&self, vector: Vector) -> f64 {
        use RectAxis::*;
        match self {
            XY => vector.z,
            XZ => vector.y,
            YZ => vector.x,
        }
    }

    fn uv(&self, vector: Vector) -> (f64, f64) {
        use RectAxis::*;
        match self {
            XY => (vector.x, vector.y),
            XZ => (vector.x, vector.z),
            YZ => (vector.y, vector.z),
        }
    }
}

pub struct Rect {
    material: Material,
    axis: RectAxis,
    p1: Point,
    p2: Point,
}

impl Rect {
    pub fn new(
        material: Material,
        axis: RectAxis,
        p1: (f64, f64),
        p2: (f64, f64),
        k: f64,
    ) -> Box<Rect> {
        let p1 = axis.point(p1, k);
        let p2 = axis.point(p2, k);
        Box::new(Rect {
            material,
            axis,
            p1,
            p2,
        })
    }
}

impl Hittable for Rect {
    fn bounding_box(&self) -> AABB {
        let offset: Vector = self.axis.point((0.0, 0.0), 0.001).coords;
        AABB::new(self.p1 - offset, self.p2 + offset)
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        let k = self.axis.k(self.p1.coords);
        let t = (k - self.axis.k(ray.origin.coords)) / self.axis.k(ray.dir);
        if !range.contains(&t) {
            return None;
        }
        let p = ray.at(t);
        const EPS: f64 = 0.0001;
        if p.x < self.p1.x - EPS
            || p.x > self.p2.x + EPS
            || p.y < self.p1.y - EPS
            || p.y > self.p2.y + EPS
            || p.z < self.p1.z - EPS
            || p.z > self.p2.z + EPS
        {
            return None;
        }

        let uv = self
            .axis
            .uv((p - self.p1).component_div(&(self.p2 - self.p1)));

        Some(Hit::new(
            ray,
            self.axis.point((0.0, 0.0), 1.0).coords,
            t,
            &self.material,
            uv,
        ))
    }
}
