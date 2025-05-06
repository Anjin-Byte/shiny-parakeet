use crate::geometry::ray::Ray;
use crate::geometry::vec3::{Color, Vec3};
use crate::hittables::cpu::hittable::HitRecord;
use crate::material::material::Material;

use crate::random_double;

pub struct Dielectric {
    refraction_index: f32,
}

impl Default for Dielectric {
    fn default() -> Self {
        Self { refraction_index: 0_f32 }
    }
}

impl Dielectric {
    pub fn new(refraction_index: f32) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
        // Use Schlick's approximation for reflectance.
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }

    pub fn refraction_index(&self) -> f32 {
        self.refraction_index
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        r_in: &Ray, 
        rec: &HitRecord, 
        attenuation: &mut Color, 
        scattered: &mut Ray
    ) -> bool {
        *attenuation = Color::new(1_f32, 1_f32, 1_f32);
        let refraction_ratio = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction: Vec3 = Vec3::unit_vector(r_in.direction);
        let cos_theta: f32 = Vec3::dot(&-unit_direction, &rec.normal).min(1_f32);
        let sin_theta: f32 = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract: bool = refraction_ratio * sin_theta > 1.0;

        let direction: Vec3 = if cannot_refract 
            || Self::reflectance(cos_theta, self.refraction_index) > random_double()  
        {
            Vec3::reflect(&unit_direction, &rec.normal)
        } else {
            Vec3::refract(&unit_direction, &rec.normal, refraction_ratio)
        };

        *scattered = Ray::new(rec.p, direction);        
        true
    }

    fn clone_box(&self) -> Box<dyn Material> {
        Box::new(Self {
            refraction_index: self.refraction_index,
        })
    }
}
