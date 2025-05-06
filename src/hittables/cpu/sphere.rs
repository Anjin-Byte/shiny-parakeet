use crate::geometry::vec3::{Vec3, Point3};
use crate::geometry::ray::Ray;
use crate::geometry::interval::Interval;

use crate::hittables::cpu::hittable::{Hittable, HitRecord};
use crate::material::material::Material;

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub mat: Box<dyn Material>
}

impl Sphere {
    pub fn new(center: Point3, radius: f32, mat: Box<dyn Material>) -> Self {
        Self { center, radius, mat }
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

        let a: f32 = r.direction.length_squared();
        let h: f32 = Vec3::dot(&r.direction, &oc);
        let c: f32 = oc.length_squared() - (self.radius * self.radius);

        let discriminant: f32 = (h * h) - (a * c);
        if discriminant < 0_f32 { return false; }
        let sqrtd: f32 = f32::sqrt(discriminant);

        // Find the nearest root that lies in the acceptable range.
        let mut root: f32 = (h - sqrtd) / a;
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

        rec.mat = self.mat.clone();

        true
    }
}