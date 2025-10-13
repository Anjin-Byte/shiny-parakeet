use super::lambertian::Lambertian;
use super::metal::Metal;
use super::dielectric::Dielectric;
use super::material::Material;

pub enum MatKind<'a> {
    Lambertian(&'a Lambertian),
    Metal (&'a Metal),
    Dielectric(&'a Dielectric),
}

impl<'a> TryFrom<&'a dyn Material> for MatKind<'a> {
    type Error = rustacuda::error::CudaError;

    fn try_from(mat: &'a dyn Material) -> Result<Self, Self::Error> {
        if let Some(l) = mat.downcast_ref::<Lambertian>() {
            Ok(MatKind::Lambertian(l))
        } else if let Some(m) = mat.downcast_ref::<Metal>() {
            Ok(MatKind::Metal(m))
        } else if let Some(d) = mat.downcast_ref::<Dielectric>() {
            Ok(MatKind::Dielectric(d))
        } else {
            unreachable!("Unsupported material type for object")
        }
    }
}