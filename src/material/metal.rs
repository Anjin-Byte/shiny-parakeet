use crate::geometry::ray::Ray;
use crate::geometry::vec3::{Color, Vec3};
use crate::hittables::hittable::HitRecord;
use crate::material::material::Material;

pub struct Metal {
    albedo: Color,
}

impl Metal {
    pub fn default() -> Self {
        Self {
            albedo: Color::default(),
        }
    }

    pub fn new(albedo: Color) -> Self {
        Self {
            albedo,
        }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray, 
        rec: &HitRecord, 
        attenuation: &mut Color, 
        scattered: &mut Ray
    ) -> bool {
        let reflected = Vec3::reflect(&r_in.direction, &rec.normal);

        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;

        true
    }


    fn clone_box(&self) -> Box<dyn Material> {
        Box::new(Self {
            albedo: self.albedo,
        })
    }
}