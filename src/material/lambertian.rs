use crate::geometry::ray::Ray;
use crate::geometry::vec3::{Color, Vec3};
use crate::hittables::hittable::HitRecord;
use crate::material::material::Material;

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
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

impl Material for Lambertian {
    #[allow(unused_variables)] // 'r_in' unused
    fn scatter(
        &self,
        r_in: &Ray, 
        rec: &HitRecord, 
        attenuation: &mut Color, 
        scattered: &mut Ray
    ) -> bool {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

        if scatter_direction.near_zero() { scatter_direction = rec.normal; }

        *scattered = Ray::new(rec.p, scatter_direction);
        *attenuation = self.albedo;

        true
    }

    fn clone_box(&self) -> Box<dyn Material> {
        Box::new(Self {
            albedo: self.albedo,
        })
    }
}