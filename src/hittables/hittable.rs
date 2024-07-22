use crate::geometry::vec3::{Vec3, Point3};
use crate::geometry::ray::Ray;
use crate::geometry::interval::Interval;

#[derive(Clone, Copy)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(
        p: Point3, 
        normal: Vec3, 
        t: f64, 
        front_face: bool
    ) -> Self {
        Self { p, normal, t, front_face }
    }

    pub(crate) fn default() -> Self {
        Self {
            p: Point3::new(0_f64, 0_f64, 0_f64),
            normal: Vec3::new(0_f64, 0_f64, 0_f64),
            t: 0_f64,
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        // Sets the hit record normal vector.
        // NOTE: the parameter `outward_normal` is assumed to have unit length.

        self.front_face = Vec3::dot(&r.direction, &outward_normal) < 0_f64;
        if self.front_face {
            self.normal = outward_normal.clone();
        } else {
            self.normal = -1_f64 * outward_normal.clone();
        }
    }
}

pub trait Hittable {
    fn hit(
        &self, 
        r: &Ray, 
        ray_t: &Interval, 
        rec: &mut HitRecord
    ) -> bool;
}