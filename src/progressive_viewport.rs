use eframe::egui::{Color32, ColorImage};

use std::{thread, u8};
use std::sync::mpsc::channel;

use crate::camera::camera::Camera;
use crate::geometry::vec3::{Point3, Color};

use crate::hittables::sphere::Sphere;
use crate::hittables::hittable_list::HittableList;

use crate::material::lambertian::Lambertian;
use crate::material::metal::Metal;

use crate::egui_app::app::init;
use crate::egui_app::command::RenderCommand;
use crate::egui_app::double_buffer::DoubleBuffer;

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

pub fn run(resolution: u32, camera_samples: u32) {
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

        let mut paused: bool = false;
        let mut accumulator: ColorImage = ColorImage::new(
            [camera.image_width as usize, camera.image_height as usize],
            Color32::BLACK,
        );

        loop {
            if !paused {
                let new_frame: ColorImage = camera.render_step_egui(&world);
                accumulator = combine_images(&accumulator, &new_frame);
                
                writer.write(|back| {
                    *back = accumulator.clone();
                });
                writer.swap();
            }
            
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    RenderCommand::Interrupt => {
                        println!("Interrupt render process!");
                        // Break out or reinitialize the render process.
                        // (You could break out of the loop or set a flag here.)
                        break;
                    }
                    RenderCommand::UpdateSample { value: _ } => todo!(),
                    RenderCommand::UpdateRes { value: _ } => todo!(),
                    RenderCommand::UpdateBounces { value: _ } => todo!(),
                    RenderCommand::UpdateAspect { value: _ } => todo!(),
                    RenderCommand::RestartRender => todo!(),
                    RenderCommand::Pause => { paused = true },
                    RenderCommand::Wake => { paused = false },
                }
            }
        }
    });


    let _ = init(tx, reader, 1.74, resolution); 
}