use crate::geometry::vec3::{Point3, Vec3};
use crate::geometry::ray::Ray;
use crate::geometry::interval::Interval;
use crate::material::lambertian::Lambertian;
use crate::material::material::Material;

use downcast_rs::Downcast;

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Box<dyn Material>,
    pub t: f32,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(
        p: Point3, 
        normal: Vec3, 
        mat: Box<dyn Material>,
        t: f32, 
        front_face: bool
    ) -> Self {
        Self { 
            p, 
            normal, 
            mat, 
            t, 
            front_face 
        }
    }

    pub(crate) fn default() -> Self {
        Self {
            p: Point3::new(0_f32, 0_f32, 0_f32),
            normal: Vec3::new(0_f32, 0_f32, 0_f32),
            mat: Box::new(Lambertian::default()),
            t: 0_f32,
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        // Sets the hit record normal vector.
        // NOTE: the parameter `outward_normal` is assumed to have unit length.

        self.front_face = Vec3::dot(&r.direction, &outward_normal) < 0_f32;
        if self.front_face {
            self.normal = outward_normal.clone();
        } else {
            self.normal = -1_f32 * outward_normal.clone();
        }
    }
}

pub trait Hittable: Downcast {
    fn hit(
        &self, 
        r: &Ray, 
        ray_t: &Interval, 
        rec: &mut HitRecord
    ) -> bool;
}

downcast_rs::impl_downcast!(Hittable);