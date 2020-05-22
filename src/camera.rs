use crate::geometry::*;
use crate::util::*;

pub struct Camera {
    origin: Point,
    lower_left: Point,
    horiz: Vector,
    vert: Vector,
    u: Vector,
    v: Vector,
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        from: Point,
        at: Point,
        up: Vector,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Camera {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let vp_height: f64 = 2.0 * h;
        let vp_width: f64 = aspect_ratio * vp_height;

        let origin = from;

        // Points from target position to camera.
        let w = (from - at).normalize();
        // Horizontal axis of the camera plane.
        let u = up.cross(&w).normalize();
        // Projects the up vector onto the plane normal to the w vector.
        let v = w.cross(&u);

        // Move the viewport focus_dist away from the camera origin
        // to allow simulating DoF.
        let horiz = focus_dist * u * vp_width;
        let vert = focus_dist * v * vp_height;

        let lower_left = origin - (horiz / 2.0) - (vert / 2.0) - focus_dist * w;

        Camera {
            origin,
            lower_left,
            horiz,
            vert,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let rd = self.lens_radius * random_in_unit_disc();
        // let rd = Vector::zeros();
        let offset: Vector = self.u * rd.x + self.v * rd.y;
        let dir = (self.lower_left + (u * self.horiz) + (v * self.vert))
            - self.origin
            - offset;
        Ray {
            origin: self.origin + offset,
            dir,
        }
    }
}
