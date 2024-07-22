use image::{ImageBuffer, Rgb};
use indicatif::ProgressBar;

use crate::geometry::ray::Ray;
use crate::geometry::vec3::{Color, Point3, Vec3};
use crate::geometry::interval::Interval;

use crate::hittables::hittable::{HitRecord, Hittable};

use crate::random_double;

pub struct Camera {
    aspect_ratio: f64,
    image_width: u32,
    image_height: u32,
    center: Point3,
    pixel_00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    samples_per_pixel: u32,
    pixel_samples_scale: f64,
}

impl Camera {
    pub(crate) fn new(aspect_ratio: f64, image_width: u32, samples: u32) -> Self {
        Self::init(aspect_ratio, image_width, samples)
    }

    pub(crate) fn render<T: Hittable>(&self, world: &T) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        // Render
        let bar = ProgressBar::new(self.image_width as u64 * self.image_height as u64);
        let img = ImageBuffer::from_fn(self.image_width, self.image_height, |i, j| {
    
            let mut pixel_color = Color::default();
            for _ in 0..self.samples_per_pixel {
                let r: Ray = self.get_ray(i, j);
                pixel_color += Self::ray_color(&r, world);
            }
            pixel_color = pixel_color * self.pixel_samples_scale;

            let intensity = Interval::new(0_f64, 0.999);
            let r: u16 = (u16::MAX as f64 * intensity.clamp(pixel_color.x())) as u16;
            let g: u16 = (u16::MAX as f64 * intensity.clamp(pixel_color.y())) as u16;
            let b: u16 = (u16::MAX as f64 * intensity.clamp(pixel_color.z())) as u16;

            bar.inc(1);
            image::Rgb([r, g, b])
        });
        bar.finish();

        img
    }

    fn init(aspect_ratio: f64, image_width: u32, samples: u32) -> Self {
        // Calculate the image height, and ensure that it's at least 1.
        let image_height: u32 = {
            let height: u32 = (image_width as f64 / aspect_ratio) as u32;
            if height < 1 {
                1
            } else {
                height
            }
        };

        let pixel_samples_scale: f64 = 1_f64 / samples as f64;

        //camera
        // Viewport widths less than one are ok since they are real valued.
        let focal_length = 1_f64;
        let viewport_height = 2_f64;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
        let camera_center = Point3::new(0_f64, 0_f64, 0_f64);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vec3::new(viewport_width, 0_f64, 0_f64);
        let viewport_v = Vec3::new(0_f64, -1_f64 * viewport_height, 0_f64);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel.
        let camera_to_viewport_vec = camera_center - Vec3::new(0_f64, 0_f64, focal_length);
        let viewport_upper_left = camera_to_viewport_vec - (0.5 * viewport_u) - (0.5 * viewport_v);
        let pixel_00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            aspect_ratio,
            image_width,
            image_height,
            center: camera_center,
            pixel_00_loc,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel: samples,
            pixel_samples_scale,
        }
    }

    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset: Vec3 = Self::sample_square();
        let pixel_sample: Vec3 = self.pixel_00_loc 
            + ((i as f64 + offset.x()) * self.pixel_delta_u) 
            + ((j as f64 + offset.y()) * self.pixel_delta_v);

        let ray_origin: Vec3 = self.center;
        let ray_direction: Vec3 = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn sample_square() -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0_f64)
    }

    fn ray_color<T: Hittable>(r: &Ray, world: &T) -> Color {
        let mut rec: HitRecord = HitRecord::default();
        if world.hit(&r, &Interval::new(0_f64, f64::INFINITY), &mut rec) {
            let direction: Vec3 = Vec3::random_on_hemisphere(&rec.normal);
            return 0.5 * Self::ray_color(&Ray::new(rec.p, direction), world)
        }

        let unit_direction = Vec3::unit_vector(r.direction);
        let a = 0.5 * (unit_direction.y() + 1.0);

        (1.0 - a) * Color::new(1_f64, 1_f64, 1_f64) + a * Color::new(0.5, 0.7, 1.0)
    }
}
