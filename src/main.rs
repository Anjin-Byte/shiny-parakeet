use geometry::vec3::Vec3;
use image::imageops::{resize, FilterType};
use image::{ImageBuffer, Luma, Rgb};
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
Book 1 finished! Congrats! :)

0) Spend time reviewing the mathamatics on paper 
to prep for your time with book 2 material.

1) Consider calcifying the code base with a refactor which 
address minor gripes and adds comments where needed. Store
the polished book 1 code base as a branch labelled as such.

2) There are all sorts of places you can take this code base
in time. GPU acceleration, noise filters, emissive material,
adaptive sampling, progressive viewport, etc..

I think it is important to keep these ideas relegated to 
specific branches until each idea is sufficiently developed.
This study of homebrew graphics is a complex one and it always
helps to stay organized.

3) "Ray Tracing: The Next Week" - Let your main branch be the one
which follows the directions of the trilogy. Integration of aformentioned
features follow a core functionality outlined in these books is completed 
to avoid issues.
*/
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut resolution: u32 = 512;
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

    let mut custom_file_tag: String = String::new();
    if let Some(arg) = args.get(3) {
        match arg.parse::<String>() {
            Ok(parsed_tag) => custom_file_tag = parsed_tag,
            Err(_) => {
                eprintln!("No custom file tags passed as arg.");
            }
        }
    }

    if PROGRESSIVE_VIEWPORT {
        run(resolution, camera_samples);
    }

    if ORIGINAL_RENDER_TO_PNG {
        let aspect_ratio: f64 = 16_f64 / 9_f64;
        let vfov = 20_f64;
        let defocus_angle = 1.4;
        let focus_dist: f64 = 3.4;

        let camera: Camera = Camera::new( // Force refresh
            defocus_angle,
            focus_dist,
            Point3::new(-2_f64, 2_f64, 1_f64),
            Point3::new(0_f64, 0_f64, -1_f64),
            Vec3::new(0_f64, 1_f64, 0_f64),
            vfov, 
            aspect_ratio, 
            resolution, 
            camera_samples
        );

        let entropy_camera: Camera = Camera::new( // Force refresh
            defocus_angle,
            focus_dist,
            Point3::new(-2_f64, 2_f64, 1_f64),
            Point3::new(0_f64, 0_f64, -1_f64),
            Vec3::new(0_f64, 1_f64, 0_f64),
            vfov, 
            aspect_ratio, 
            resolution / 4, 
            camera_samples
        );

        // (94,187,161)
        let material_ground: Lambertian = Lambertian::new(Color::new(0.419, 0.400, 0.776));
        //let material_ground_metal: Metal = Metal::new(Color::new(0.419, 0.400, 0.776), 0_f64);
        let material_center: Lambertian = Lambertian::new(Color::new(94.0/256.0, 187.0/256.0, 161.0/256.0));
        //let material_center: Lambertian = Lambertian::new(Color::new(0.9, 0.9, 0.9));
        let material_left: Dielectric = Dielectric::new(1.50);
        let material_bubble: Dielectric = Dielectric::new(1_f64 / 1.50);
        let material_right: Metal = Metal::new(Color::new(0.6, 0.6, 0.7), 0.02);
        

        let empty_material_ground: Lambertian = Lambertian::new(Color::new(0.419, 0.400, 0.776));
        let mut empty_world: HittableList = HittableList::new();
        empty_world.add(Box::new(Sphere::new(
            Point3::new(0_f64, -100.5, -1_f64),
            100_f64,
            Box::new(material_ground)
        )));

        let mut world: HittableList = HittableList::new();
        
        world.add(Box::new(Sphere::new(
            Point3::new(0_f64, -100.5, -1_f64),
            100_f64,
            Box::new(empty_material_ground)
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

        //let img: ImageBuffer<Rgb<u16>, Vec<u16>> = camera.render(&world);

/*         let img_adaptive = camera.entropy_adaptive_render(
            &world, 
            8, 
            100, 
            500, 
            2.0, 
            0.05, 
            false
        );  */

        let heatmap_entropy = camera.entropy_heatmap_temporal(
            &world, 
            8, 
            0.5,
            24
        );

        let final_image = camera.pre_compute_adaptive_render(
            &world,
            &heatmap_entropy,
            64,
            256,
            0.85,
        );

       // let filter = FilterType::Triangle;
        //let filtered_img = resize(&final_image, camera.image_height, camera.image_height, filter);

        let img_name = format!(
            "out/{2}/{1:.prec$}_{3}_{0}.png", 
            custom_file_tag,
            aspect_ratio, 
            resolution, 
            camera_samples,
            prec = 2,
        );

        let img_adaptive_name = format!(
            "out/{2}/{1:.prec$}_{3}_{0}_entropy_exp.png", 
            custom_file_tag,
            aspect_ratio, 
            resolution, 
            camera_samples,
            prec = 2,
        );

        let path = Path::new(&img_name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
    
        let adaptive_path = Path::new(&img_adaptive_name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
/*                           
        if let Err(e) = img.save(&img_name) {
            eprintln!("Failed to save image: {}", e);
        } else {
            println!("Image successfully saved to: {:#?}", path);
        } */

        if let Err(e) = final_image.save("heatmap.png") {
            eprintln!("Failed to save image: {}", e);
        } else {
            println!("Image successfully saved to: {:#?}", path);
        } 
    }
}
