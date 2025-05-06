use crate::geometry::ray::Ray;
use crate::geometry::vec3::{Color, Vec3};
use crate::hittables::cpu::hittable::HitRecord;
use crate::material::material::Material;

pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

impl Metal {
    pub fn default() -> Self {
        Self {
            albedo: Color::default(),
            fuzz: 0_f32,
        }
    }

    pub fn new(albedo: Color, fuzz: f32) -> Self {
        let fuzz: f32 = if fuzz < 1_f32 { fuzz } else { 1_f32 };

        Self {
            albedo,
            fuzz,
        }
    }

    pub fn albedo(&self) -> Color {
        self.albedo
    }

    pub fn fuzz(&self) -> f32 {
        self.fuzz
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
        let mut reflected = Vec3::reflect(&r_in.direction, &rec.normal);
        reflected = Vec3::unit_vector(reflected) + (self.fuzz * Vec3::random_unit_vector());
        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;

        Vec3::dot(&scattered.direction, &rec.normal) > 0_f32
    }


    fn clone_box(&self) -> Box<dyn Material> {
        Box::new(Self {
            albedo: self.albedo,
            fuzz: self.fuzz,
        })
    }
}