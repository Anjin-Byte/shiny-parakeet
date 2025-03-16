use eframe::egui::{Color32, ColorImage};
use image::{ImageBuffer, Rgb};
use rand::Rng;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs, thread, u8};
use std::sync::mpsc::channel;
use std::path::Path;
use std::f64::consts::PI;

pub mod camera;
pub mod geometry;
pub mod hittables;
pub mod material;
pub mod egui_app;
pub mod test;

use crate::camera::camera::Camera;
use crate::geometry::vec3::{Point3, Color};

use crate::hittables::sphere::Sphere;
use crate::hittables::hittable_list::HittableList;

use crate::material::lambertian::Lambertian;
use crate::material::metal::Metal;

use crate::egui_app::app::init;
use crate::egui_app::command::RenderCommand;
use crate::egui_app::double_buffer::DoubleBuffer;

const OLD_RENDER_TO_PNG: bool = false;
const PROGRESSIVE_VIEWPORT: bool = true;

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

fn combine_images(img1: &ColorImage, img2: &ColorImage) -> ColorImage {
    assert_eq!(img1.size, img2.size, "Images must be the same size");
    
    let pixels = img1.pixels.iter().zip(&img2.pixels)
        .map(|(p1, p2)| {
            let r = ((p1.r() as u16) + (p2.r() as u16)) / 2;
            let g = ((p1.g() as u16) + (p2.g() as u16)) / 2;
            let b = ((p1.b() as u16) + (p2.b() as u16)) / 2;
            Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, u8::MAX)
        })
        .collect();
        
    ColorImage {
        size: img1.size,
        pixels,
    }
}

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

    let aspect_ratio: f64 = 16_f64 / 9_f64;
    let camera: Camera = Camera::new(aspect_ratio, resolution, camera_samples);

    let (tx, rx) = channel::<RenderCommand>();
    let db = DoubleBuffer::new(
        ColorImage::new([camera.image_width as usize, camera.image_height as usize], Color32::WHITE), 
        ColorImage::new([camera.image_width as usize, camera.image_height as usize], Color32::WHITE)
    );
    let (reader, mut writer) = db.split();
    
    thread::spawn(move || {
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

        //let img: ImageBuffer<Rgb<u16>, Vec<u16>> = camera.render(&world);

        if OLD_RENDER_TO_PNG {
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
        
            /*          
            if let Err(e) = img.save(&img_name) {
                eprintln!("Failed to save image: {}", e);
            } else {
                println!("Image successfully saved to: {:#?}", path);
            } 
            */
        }

        let mut paused: bool = false;
        let mut accumulator: ColorImage = ColorImage::new(
            [camera.image_width as usize, camera.image_height as usize],
            Color32::BLACK,
        );

        loop {
            //let mut current_render = ColorImage::new([camera.image_width as usize, camera.image_height as usize], Color32::BLACK);
            if !paused {
                //println!("Rendering");
                let new_frame: ColorImage = camera.render_step_egui(&world);
                accumulator = combine_images(&accumulator, &new_frame);
  
                writer.write(|back| {
                    *back = accumulator.clone();
                });
                writer.swap();
                println!("Rendered new frame")
            }
            
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    RenderCommand::Interrupt => {
                        println!("Interrupt render process!");
                        // Break out or reinitialize the render process.
                        // (You could break out of the loop or set a flag here.)
                        break;
                    }
                    RenderCommand::UpdateSample { value } => todo!(),
                    RenderCommand::UpdateRes { value } => todo!(),
                    RenderCommand::UpdateBounces { value } => todo!(),
                    RenderCommand::UpdateAspect { value } => todo!(),
                    RenderCommand::RestartRender => todo!(),
                    RenderCommand::Pause => { paused = true },
                    RenderCommand::Wake => { paused = false },
                }
            }
        }

        println!("Thread 1 finished!");
    });

    if PROGRESSIVE_VIEWPORT { 
        let _ = init(tx, reader, 1.74, resolution); 
    }

    println!("Main thread finished!");
}
