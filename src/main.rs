use image::{ImageBuffer, Rgb};
use rand::Rng;

use std::{env, fs};
use std::path::Path;
use std::f64::consts::PI;

pub mod camera;
pub mod geometry;
pub mod hittables;
pub mod material;
pub mod egui_app;
pub mod progressive_viewport;
pub mod test;

use crate::camera::camera::Camera;
use crate::geometry::vec3::{Point3, Color};

use crate::hittables::sphere::Sphere;
use crate::hittables::hittable_list::HittableList;

use crate::material::lambertian::Lambertian;
use crate::material::metal::Metal;
use crate::material::dielectric::Dielectric;

use progressive_viewport::run;

const ORIGINAL_RENDER_TO_PNG: bool = true;
const PROGRESSIVE_VIEWPORT: bool = false;

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

/* TODO
Finish book 1 first.

Then focus on the refactor to enable GUI dev. Use refactor to refamiliarize yourself with 
book 1 concepts. Once you refactor there will be divergences with book code 
and your own which may be significant. You must have good understanding of book 1
logic in order to continue implementing concepts in book 2-3. 
*/
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut resolution: u32 = 1000;
    if let Some(arg) = args.get(1) {
        match arg.parse::<u32>() {
            Ok(parsed_res) => resolution = parsed_res,
            Err(_) => {
                eprintln!("Invalid resolution provided, using default: {}", resolution);
            }
        }
    }

    let mut camera_samples: u32 = 100;
    if let Some(arg) = args.get(2) {
        match arg.parse::<u32>() {
            Ok(parsed_sample_rate) => camera_samples = parsed_sample_rate,
            Err(_) => {
                eprintln!("Invalid sample rate provided, using default: {}", camera_samples);
            }
        }
    }

    if PROGRESSIVE_VIEWPORT {
        run(resolution, camera_samples);
    }

    if ORIGINAL_RENDER_TO_PNG {
        let aspect_ratio: f64 = 16_f64 / 9_f64;
        let vfov: f64 = 100_f64;
        let camera: Camera = Camera::new(vfov, aspect_ratio, resolution, camera_samples);
    
        let material_ground: Lambertian = Lambertian::new(Color::new(0.8, 0.8, 0_f64));
        //let material_ground_metal: Metal = Metal::new(Color::new(0.8, 0.8, 0.8), 0_f64);
        let material_center: Lambertian = Lambertian::new(Color::new(0.1, 0.2, 0.5));
        
        let material_left: Dielectric = Dielectric::new(1.50);
        let material_bubble: Dielectric = Dielectric::new(1_f64 / 1.50);
        let material_right: Metal = Metal::new(Color::new(0.8, 0.6, 0.2), 0_f64);
    
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
            Point3::new(-1_f64, 0_f64, -1_f64),
            0.4,
            Box::new(material_bubble)
        )));

        world.add(Box::new(Sphere::new(
            Point3::new(1_f64, 0_f64, -1_f64),
            0.5,
            Box::new(material_right)
        )));

        let img: ImageBuffer<Rgb<u16>, Vec<u16>> = camera.render(&world);

        let img_name = format!(
            "out/{1}/{0:.prec$}_{2}.png", 
            aspect_ratio, 
            resolution, 
            camera_samples,
            prec = 2,
        );

        let path = Path::new(&img_name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
    
                  
        if let Err(e) = img.save(&img_name) {
            eprintln!("Failed to save image: {}", e);
        } else {
            println!("Image successfully saved to: {:#?}", path);
        } 
    }
}
