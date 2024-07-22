use crate::geometry::vec3::{Vec3, Point3};
use crate::geometry::ray::Ray;
use crate::geometry::interval::Interval;

use crate::hittables::hittable::{Hittable, HitRecord};

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl Hittable for Sphere {
    fn hit(
        &self, 
        r: &Ray, 
        ray_t: &Interval,
        rec: &mut HitRecord
    ) -> bool {
        let oc: Vec3 = self.center - r.origin;

        let a: f64 = r.direction.length_squared();
        let h: f64 = Vec3::dot(&r.direction, &oc);
        let c: f64 = oc.length_squared() - (self.radius * self.radius);

        let discriminant: f64 = (h * h) - (a * c);
        if discriminant < 0_f64 { return false; }
        let sqrtd: f64 = f64::sqrt(discriminant);

        // Find the nearest root that lies in the acceptable range.
        let mut root: f64 = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);

        let outward_normal: Vec3 = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, &outward_normal);

        true
    }
}