use image::{ImageBuffer, Rgb};
use rand::Rng;

use rustacuda::{CudaFlags, context::{Context, ContextFlags}};
use rustacuda::error::CudaResult;
use rustacuda::prelude::*; 

use std::{env, fs};
use std::path::Path;
use std::f32::consts::PI;

pub mod camera;
pub mod geometry;
pub mod hittables;
pub mod material;
pub mod egui_app;
pub mod progressive_viewport;
pub mod test;

use crate::camera::camera::Camera;
use crate::geometry::vec3::{Vec3, Point3, Color};

use crate::hittables::cpu::sphere::Sphere;
use crate::hittables::cpu::hittable_list::HittableList;
use crate::hittables::gpu::gpu::{GPUScene/* , MaterialPrimitive, SpherePrimitive */};

use crate::material::lambertian::Lambertian;
use crate::material::metal::Metal;
use crate::material::dielectric::Dielectric;

use progressive_viewport::run;

#[macro_use]
extern crate rustacuda;

#[macro_use]
extern crate rustacuda_derive;
extern crate rustacuda_core;

const ORIGINAL_RENDER_TO_PNG: bool = true;
const PROGRESSIVE_VIEWPORT: bool = false;

#[allow(dead_code)]
fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180_f32
}

fn random_double() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..=1.0)
}

fn random_double_range(min: f32, max: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::process::Command;
    use std::path::PathBuf;
    use std::error::Error;
    use std::ffi::CString;

    use rustacuda::prelude::*;
    use rustacuda::memory::DeviceBox;
    use rustacuda::function::{GridSize, BlockSize};

    #[test]
    fn test_cuda_add() -> Result<(), Box<dyn Error>> {
        // — Initialize the CUDA API
        rustacuda::init(CudaFlags::empty())?;

        // — Pick the first device and create a context
        let device = Device::get_device(0)?;
        let _context = Context::create_and_push(
            ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO,
            device,
        )?;

        // — Load the PTX module
        let ptx = CString::new(include_str!("../resources/add/add.ptx"))?;
        let module = Module::load_from_string(&ptx)?;

        // — Create a non-blocking stream
        let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

        // — Allocate three floats on the device
        let mut x = DeviceBox::new(&10.0f32)?;
        let mut y = DeviceBox::new(&20.0f32)?;
        let mut result = DeviceBox::new(&0.0f32)?;

        // — Launch `sum(x, y, result, 1)` on 1 block × 1 thread
        unsafe {
            launch!(module.sum<<<1, 1, 0, stream>>>(
                x.as_device_ptr(),
                y.as_device_ptr(),
                result.as_device_ptr(),
                1u32
            ))?;
        }
        stream.synchronize()?;

        // — Copy the result back and assert
        let mut host_val = 0.0f32;
        result.copy_to(&mut host_val)?;
        assert_eq!(host_val, 30.0f32);

        Ok(())
    }

    #[test]
    fn rustacuda_init_and_device_query() -> Result<(), Box<dyn Error>> {
        rustacuda::init(CudaFlags::empty())?;
        let count = Device::num_devices()?;

        assert!(
            count >= 1,
            "Found zero CUDA-capable devices; is your driver/toolkit installed?"
        );

        let device0 = Device::get_device(0)?;
        println!("CUDA device[0]: {}", device0.name()?);

        Ok(())
    }

    #[test]
    fn rustacuda_gradient_render() -> Result<(), Box<dyn Error>> {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let cu = manifest_dir.join("resources/gradient").join("render.cu");
        let ptx_path = manifest_dir
        .join("resources")
        .join("gradient")
        .join("render.ptx");
        println!("Loading PTX from {:?}", &ptx_path);

        let output = Command::new("nvcc")
            .arg("--ptx")
            .arg("-o")
            .arg(&ptx_path)
            .arg("-arch=compute_61")
            .arg(cu)
            .arg("-Wno-deprecated-gpu-targets")
            .output()?;

        println!("{:?}", &output);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("\n\nnvcc failed with code {}:\n{}\n",
                   output.status, stderr);
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("\n\nnvcc successfully compiled .cu {}:\n{}\n",
                   output.status, stdout);
        }

        println!("Compile occured");

        // Otherwise, let Cargo know the PTX is an output dependency (optional)
        //println!("cargo:rerun-if-changed={}", cu.display());

        rustacuda::init(CudaFlags::empty())?;

        let device = Device::get_device(0)?;
        let _context = Context::create_and_push(
            ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO,
            device,
        )?;

        println!("device found");

        //let ptx_str = include_str!("../resources/gradient/render.cubin");
        //println!("PTX:\n{}", &ptx_str[..200.min(ptx_str.len())]);

        let ptx_src = fs::read_to_string(&ptx_path)?;
        let ptx = CString::new(ptx_src)?;
        let module = Module::load_from_string(&ptx)?;

        let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

        println!("kernel start");
        // Set up frame buffer parameters
        const MAX_X: u32 = 4000;
        const MAX_Y: u32 = 4000;
        let num_pixels = (MAX_X * MAX_Y * 3) as usize; // 3 floats per pixel (RGB)
        // Allocate and zero-initialize GPU buffer
        let mut dev_fb = DeviceBuffer::from_slice(&vec![0u16; num_pixels])?;

        let block_x = 8;
        let block_y = 8;
        let grid_x = (MAX_X + block_x - 1) / block_x;
        let grid_y = (MAX_Y + block_y - 1) / block_y;

        let grid  = GridSize::xyz(grid_x,  grid_y,  1);
        let block = BlockSize::xyz(block_x, block_y, 1);

        unsafe {
            launch!(
                module.gradient<<<grid, block, 0, stream>>>(
                    dev_fb.as_device_ptr(),
                    MAX_X as i32,
                    MAX_Y as i32
                )
            )?;
        }
    
        // Wait for GPU work to finish
        stream.synchronize()?;

        println!("kernel end - start copy");
        let mut host_fb = vec![0u16; num_pixels];
        dev_fb.copy_to(&mut host_fb)?;
        println!("end copy - start write");

/*         let flat_u16: Vec<u16> = host_fb
            .iter()
            .map(|&v| {
                // scale [0.0,1.0] → [0, 65535], then clamp (just in case)
                let v = (v * u16::MAX as f32).round();
                v.max(0.0).min(u16::MAX as f32) as u16
            })
            .collect();
 */
        // 3) Build your ImageBuffer – from_raw takes (width, height, pixel_data):
        //    Pixel order in the Vec must be R,G,B,R,G,B, … row by row.
        let img: ImageBuffer<Rgb<u16>, Vec<u16>> =
            ImageBuffer::from_raw(MAX_X, MAX_Y, host_fb)
                .expect("buffer size does not match dimensions");

        // Now you can save it (for example) as a PNG with 16-bit per channel:
        img.save("test_render.png")?;

        Ok(())
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

    let mut custom_file_tag: String = String::new();
    if let Some(arg) = args.get(3) {
        match arg.parse::<String>() {
            Ok(parsed_tag) => custom_file_tag = parsed_tag,
            Err(_) => {
                eprintln!("No custom file tags passed as arg.");
            }
        }
    }

    let _ = rustacuda::init(CudaFlags::empty());
    let device = Device::get_device(0);
    let _context = Context::create_and_push(
        ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO,
        device.unwrap(), // thread may panic here
    );
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None);

    if PROGRESSIVE_VIEWPORT {
        run(resolution, camera_samples);
    }

    if ORIGINAL_RENDER_TO_PNG {
        let aspect_ratio: f32 = 16_f32 / 9_f32;
        let vfov = 20_f32;
        let defocus_angle = 1.4;
        let focus_dist: f32 = 3.4;

        let camera: Camera = Camera::new( // Force refresh
            defocus_angle,
            focus_dist,
            Point3::new(-2_f32, 2_f32, 1_f32),
            Point3::new(0_f32, 0_f32, -1_f32),
            Vec3::new(0_f32, 1_f32, 0_f32),
            vfov, 
            aspect_ratio, 
            resolution, 
            camera_samples
        );
        // (94,187,161)
        let material_ground: Lambertian = Lambertian::new(Color::new(0.419, 0.400, 0.776));
        //let material_ground_metal: Metal = Metal::new(Color::new(0.419, 0.400, 0.776), 0_f32);
        let material_center: Lambertian = Lambertian::new(Color::new(94.0/256.0, 187.0/256.0, 161.0/256.0));
        //let material_center: Lambertian = Lambertian::new(Color::new(0.9, 0.9, 0.9));
        let material_left: Dielectric = Dielectric::new(1.50);
        let material_bubble: Dielectric = Dielectric::new(1_f32 / 1.50);
        let material_right: Metal = Metal::new(Color::new(0.6, 0.6, 0.7), 0.02);
    
        let mut world: HittableList = HittableList::new();
        
        world.add(Box::new(Sphere::new(
            Point3::new(0_f32, -100.5, -1_f32),
            100_f32,
            Box::new(material_ground)
        )));
        world.add(Box::new(Sphere::new(
            Point3::new(0_f32, 0_f32, -1.2),
            0.5,
            Box::new(material_center)
        )));
    
        world.add(Box::new(Sphere::new(
            Point3::new(-1_f32, 0_f32, -1_f32),
            0.5,
            Box::new(material_left)
        )));
        world.add(Box::new(Sphere::new(
            Point3::new(-1_f32, 0_f32, -1_f32),
            0.4,
            Box::new(material_bubble)
        )));

        world.add(Box::new(Sphere::new(
            Point3::new(1_f32, 0_f32, -1_f32),
            0.5,
            Box::new(material_right)
        )));


        let gpu_scene = GPUScene::try_from(&world).unwrap(); // !! will panic if world not supported
        println!("{:?}", gpu_scene);

        let img: ImageBuffer<Rgb<u16>, Vec<u16>> = camera.render(&world);

        let img_name = format!(
            "out/{2}/{1:.prec$}_{3}_{0}.png", 
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
    
                  
        if let Err(e) = img.save(&img_name) {
            eprintln!("Failed to save image: {}", e);
        } else {
            println!("Image successfully saved to: {:#?}", path);
        } 
    }
}
