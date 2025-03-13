use image::{ImageBuffer, Rgb};
use rand::Rng;

use std::env;
use std::path::Path;
use std::f64::consts::PI;

pub mod camera;
pub mod geometry;
pub mod hittables;
pub mod material;
pub mod test;

use crate::camera::camera::Camera;
use crate::geometry::vec3::{Point3, Color};

use crate::hittables::sphere::Sphere;
use crate::hittables::hittable_list::HittableList;

use crate::material::lambertian::Lambertian;
use crate::material::metal::Metal;

#[allow(dead_code)]
fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180_f64
}

fn random_double() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..=1.0)
}

fn random_double_range(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut resolution: u32 = 128;
    if let Some(arg) = args.get(1) {
        match arg.parse::<u32>() {
            Ok(parsed_res) => resolution = parsed_res,
            Err(_) => {
                eprintln!("Invalid resolution provided, using default: {}", resolution);
            }
        }
    }

    let mut camera_samples: u32 = 128;
    if let Some(arg) = args.get(2) {
        match arg.parse::<u32>() {
            Ok(parsed_sample_rate) => camera_samples = parsed_sample_rate,
            Err(_) => {
                eprintln!("Invalid sample rate provided, using default: {}", camera_samples);
            }
        }
    }

    let material_ground: Lambertian = Lambertian::new(Color::new(0.8, 0.8, 0_f64));
    let material_center: Lambertian = Lambertian::new(Color::new(0.1, 0.2, 0.5));

    let material_left: Metal = Metal::new(Color::new(0.8, 0.8, 0.8));
    let material_right: Metal = Metal::new(Color::new(0.8, 0.6, 0.2));

    let mut world: HittableList = HittableList::new();
    
    world.add(Box::new(Sphere::new(
        Point3::new(0_f64, -100.5, -1_f64),
        100_f64,
        Box::new(material_ground)
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0_f64, 0_f64, -1.2),
        0.5,
        Box::new(material_center)
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(-1_f64, 0_f64, -1_f64),
        0.5,
        Box::new(material_left)
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(1_f64, 0_f64, -1_f64),
        0.5,
        Box::new(material_right)
    )));

    let aspect_ratio: f64 = 16_f64 / 16_f64;
    let camera: Camera = Camera::new(aspect_ratio, resolution, camera_samples);
    let img: ImageBuffer<Rgb<u16>, Vec<u16>> = camera.render(&world);

    let img_name = format!(
        "out/{1}/{:.prec$}_{1}_{2}.png", 
        aspect_ratio, 
        resolution, 
        camera_samples,
        prec = 2,
    );
    let path = Path::new(&img_name);

    let _ = img.save(path).unwrap_or(());
}
