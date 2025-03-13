use crate::geometry::ray::Ray;
use crate::geometry::vec3::Color;
use crate::hittables::hittable::HitRecord;

pub trait Material {
    fn scatter(
        &self,
        r_in: &Ray, 
        rec: &HitRecord, 
        attenuation: &mut Color, 
        scattered: &mut Ray
    ) -> bool;

    fn clone_box(&self) -> Box<dyn Material>;
}

impl Clone for Box<dyn Material> {
    fn clone(&self) -> Box<dyn Material> {
        self.clone_box()
    }
}