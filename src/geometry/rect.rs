use super::*;
use crate::material::Material;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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

    fn unit(&self) -> Unit<Vector> {
        use RectAxis::*;
        match self {
            XY => Vector::z_axis(),
            XZ => Vector::y_axis(),
            YZ => Vector::x_axis(),
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
    material: Arc<Material>,
    axis: RectAxis,
    p1: Point,
    p2: Point,
    area: f64,
}

impl Rect {
    pub fn new(
        material: Arc<Material>,
        axis: RectAxis,
        (a1, b1): (f64, f64),
        (a2, b2): (f64, f64),
        k: f64,
    ) -> Arc<Rect> {
        let (a1, b1, a2, b2) = (
            f64::min(a1, a2),
            f64::min(b1, b2),
            f64::max(a1, a2),
            f64::max(b1, b2),
        );
        let p1 = axis.point((a1, b1), k);
        let p2 = axis.point((a2, b2), k);
        let area = ((a2 - a1) * (b2 - b1)).abs();
        Arc::new(Rect {
            material,
            axis,
            p1,
            p2,
            area,
        })
    }
}

impl Hittable for Rect {
    fn is_light(&self) -> bool {
        match self.material.as_ref() {
            Material::Emission(..) => true,
            _ => false,
        }
    }

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

        Some(Hit::new(ray, self.axis.unit(), t, &self.material, uv))
    }

    fn pdf(&self, ray: Ray) -> f64 {
        match self.hit(ray, 0.0001..f64::INFINITY) {
            None => 0.0,
            Some(hit) => {
                let norm_squared = hit.t * hit.t * ray.dir.norm_squared();
                let cos = (ray.dir.dot(&hit.normal) / ray.dir.norm()).abs();
                norm_squared / (cos * self.area)
            }
        }
    }

    fn random(&self, origin: Point) -> Vector {
        const EPS: f64 = 0.0001;
        let rand_point = Point::new(
            random_range(self.p1.x - EPS..self.p2.x + EPS),
            random_range(self.p1.y - EPS..self.p2.y + EPS),
            random_range(self.p1.z - EPS..self.p2.z + EPS),
        );
        rand_point - origin
    }

    fn emit(&self) -> (Ray, Unit<Vector>, Color) {
        const EPS: f64 = 0.0001;
        let dir = -1.0 * *self.axis.unit() + random_unit_vector();
        let rand_point = Point::new(
            random_range(self.p1.x - EPS..self.p2.x + EPS),
            random_range(self.p1.y - EPS..self.p2.y + EPS),
            random_range(self.p1.z - EPS..self.p2.z + EPS),
        );
        let ray = Ray {
            origin: rand_point,
            // TODO: Properly use normals here.
            dir,
        };
        let normal = Unit::new_normalize(dir);
        (
            ray,
            normal,
            self.material.emitted(&Hit::new(
                ray,
                Unit::new_normalize(dir),
                0.0,
                &self.material,
                (0.0, 0.0),
            )),
        )
    }
}
