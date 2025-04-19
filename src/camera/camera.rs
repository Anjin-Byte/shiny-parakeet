use std::u8;

use eframe::egui::{Color32, ColorImage};
use image::{ImageBuffer, Rgb};
use indicatif::ProgressBar;

use crate::geometry::interval::Interval;
use crate::geometry::ray::Ray;
use crate::geometry::vec3::{Color, Point3, Vec3};

use crate::hittables::hittable::{HitRecord, Hittable};

use crate::{degrees_to_radians, random_double};

pub struct Camera {
    #[allow(dead_code)] // 'aspect_ratio' unused
    aspect_ratio: f64,
    pub image_width: u32,
    pub image_height: u32,
    center: Point3,
    pixel_00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    samples_per_pixel: u32,
    pixel_samples_scale: f64,
    max_depth: u32,
    #[allow(dead_code)] // 'vfov' unused
    vfov: f64,
    lookfrom: Point3,
    lookat: Point3,
    vup: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    pub defocus_angle: f64,
    pub focus_dist: f64,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub(crate) fn new(
        defocus_angle: f64,
        focus_dist: f64,
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        image_width: u32,
        samples: u32,
    ) -> Self {
        Self::init(
            defocus_angle,
            focus_dist,
            lookfrom,
            lookat,
            vup,
            vfov,
            aspect_ratio,
            image_width,
            samples,
        )
    }

    fn luminance(pixel: &Rgb<u16>) -> f64 {
        let r = pixel[0] as f64 / u16::MAX as f64;
        let g = pixel[1] as f64 / u16::MAX as f64;
        let b = pixel[2] as f64 / u16::MAX as f64;
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn samples_from_luminance(luminance: f64, min_samples: u32, max_samples: u32, gamma: f64) -> u32 {
        let lum = luminance.clamp(0.0, 1.0);
        let samples = min_samples as f64 + (max_samples - min_samples) as f64 * lum.powf(gamma);
        samples.round() as u32
    }    

    pub fn sobel_filter(
        image: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        threshold: f64,
    ) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let (width, height) = image.dimensions();
        let mut edge_img = ImageBuffer::new(width, height);

        // Helper to safely access pixel luminance with boundary checks
        let get_lum = |x: i32, y: i32| {
            if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 {
                0.0
            } else {
                Self::luminance(&image.get_pixel(x as u32, y as u32))
            }
        };

        for y in 0..height as i32 {
            for x in 0..width as i32 {
                // Sobel kernels
                let gx = -1.0 * get_lum(x - 1, y - 1)
                    + 1.0 * get_lum(x + 1, y - 1)
                    + -2.0 * get_lum(x - 1, y)
                    + 2.0 * get_lum(x + 1, y)
                    + -1.0 * get_lum(x - 1, y + 1)
                    + 1.0 * get_lum(x + 1, y + 1);

                let gy = -1.0 * get_lum(x - 1, y - 1)
                    + -2.0 * get_lum(x, y - 1)
                    + -1.0 * get_lum(x + 1, y - 1)
                    + 1.0 * get_lum(x - 1, y + 1)
                    + 2.0 * get_lum(x, y + 1)
                    + 1.0 * get_lum(x + 1, y + 1);

                // Edge magnitude approximation (faster than sqrt(gx^2 + gy^2))
                let magnitude = gx.abs() + gy.abs();

                let edge_strength = if magnitude > threshold { u16::MAX } else { 0 };

                // Write edge detection result as a grayscale pixel (white edges)
                edge_img.put_pixel(
                    x as u32,
                    y as u32,
                    Rgb([edge_strength, edge_strength, edge_strength]),
                );
            }
        }

        edge_img
    }

    fn linear_to_gamma(linear_component: f64) -> f64 {
        if linear_component > 0_f64 {
            return f64::sqrt(linear_component);
        }

        0_f64
    }

    pub fn egui_image_from_fn<F>(width: u32, height: u32, mut f: F) -> ColorImage
    where
        F: FnMut(u32, u32) -> Color32,
    {
        let mut img = ColorImage::new([width as usize, height as usize], Color32::WHITE);
        for (i, p) in img.pixels.iter_mut().enumerate() {
            let x: u32 = i as u32 % width;
            let y: u32 = i as u32 / width;

            *p = f(x, y);
        }
        img
    }

    pub(crate) fn render_step_egui(&self, world: &dyn Hittable) -> ColorImage {
        let img = Self::egui_image_from_fn(self.image_width, self.image_height, |i, j| {
            let r: Ray = self.get_ray(i, j);
            let pixel_color = Self::ray_color(&r, self.max_depth, world);

            let r_gamma: f64 = Self::linear_to_gamma(pixel_color.x());
            let g_gamma: f64 = Self::linear_to_gamma(pixel_color.y());
            let b_gamma: f64 = Self::linear_to_gamma(pixel_color.z());

            let intensity = Interval::new(0_f64, 0.999);
            let r: u8 = (u8::MAX as f64 * intensity.clamp(r_gamma)) as u8;
            let g: u8 = (u8::MAX as f64 * intensity.clamp(g_gamma)) as u8;
            let b: u8 = (u8::MAX as f64 * intensity.clamp(b_gamma)) as u8;

            Color32::from_rgba_unmultiplied(r, g, b, u8::MAX)
        });

        img
    }

    /* render step returning 'image' crate ImageBuffer type
    pub(crate) fn render_step(&self, world: &dyn Hittable) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let img = ImageBuffer::from_fn(self.image_width, self.image_height, |i, j| {
            //let mut pixel_color = Color::default();
            let r: Ray = self.get_ray(i, j);
            let pixel_color = Self::ray_color(&r, self.max_depth, world);

            //pixel_color = pixel_color * self.pixel_samples_scale;

            let r_gamma: f64 = Self::linear_to_gamma(pixel_color.x());
            let g_gamma: f64 = Self::linear_to_gamma(pixel_color.y());
            let b_gamma: f64 = Self::linear_to_gamma(pixel_color.z());

            let intensity = Interval::new(0_f64, 0.999);
            let r: u16 = (u16::MAX as f64 * intensity.clamp(r_gamma)) as u16;
            let g: u16 = (u16::MAX as f64 * intensity.clamp(g_gamma)) as u16;
            let b: u16 = (u16::MAX as f64 * intensity.clamp(b_gamma)) as u16;

            image::Rgb([r, g, b])
        });

        img
    }
    */
    pub(crate) fn render(&self, world: &dyn Hittable) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let bar = ProgressBar::new(self.image_width as u64 * self.image_height as u64);
        let img = ImageBuffer::from_fn(self.image_width, self.image_height, |i, j| {
            let mut pixel_color = Color::default();
            for _ in 0..self.samples_per_pixel {
                let r: Ray = self.get_ray(i, j);
                pixel_color += Self::ray_color(&r, self.max_depth, world);
            }
            pixel_color = pixel_color * self.pixel_samples_scale;

            let r_gamma: f64 = Self::linear_to_gamma(pixel_color.x());
            let g_gamma: f64 = Self::linear_to_gamma(pixel_color.y());
            let b_gamma: f64 = Self::linear_to_gamma(pixel_color.z());

            let intensity = Interval::new(0_f64, 0.999);
            let r: u16 = (u16::MAX as f64 * intensity.clamp(r_gamma)) as u16;
            let g: u16 = (u16::MAX as f64 * intensity.clamp(g_gamma)) as u16;
            let b: u16 = (u16::MAX as f64 * intensity.clamp(b_gamma)) as u16;

            bar.inc(1);
            image::Rgb([r, g, b])
        });
        bar.finish();

        img
    }

    pub(crate) fn map_highres_to_lowres(
        high_x: u32, high_y: u32,
        high_width: u32, high_height: u32,
        low_width: u32, low_height: u32,
    ) -> (u32, u32) {
        let scale_x = high_width as f64 / low_width as f64;
        let scale_y = high_height as f64 / low_height as f64;
    
        let low_x = (high_x as f64 / scale_x).floor() as u32;
        let low_y = (high_y as f64 / scale_y).floor() as u32;
    
        (low_x.min(low_width - 1), low_y.min(low_height - 1))
    }

    pub(crate) fn render_adaptive(
        &self, 
        world: &dyn Hittable,
        min_samples: u32,
        max_samples: u32,
        gamma: f64,
        edge_filter_img: &ImageBuffer<Rgb<u16>, Vec<u16>>,
    ) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let (low_width, low_height) = edge_filter_img.dimensions();

        let bar = ProgressBar::new(self.image_width as u64 * self.image_height as u64);
        let adaptive_img = ImageBuffer::from_fn(self.image_width, self.image_height, |i, j| {
            let (low_i, low_j) = Self::map_highres_to_lowres(
                i, j, 
                self.image_width, self.image_height, 
                low_width, low_height,
            );

            let luminance = Self::luminance(edge_filter_img.get_pixel(low_i, low_j));
            let samples = Self::samples_from_luminance(
                luminance, 
                min_samples, 
                max_samples, 
                gamma
            );

            let mut pixel_color = Color::default();
            for _ in 0..samples {
                let r: Ray = self.get_ray(i, j);
                pixel_color += Self::ray_color(&r, self.max_depth, world);
            }
            pixel_color = pixel_color * (1_f64 / samples as f64);

            let r_gamma: f64 = Self::linear_to_gamma(pixel_color.x());
            let g_gamma: f64 = Self::linear_to_gamma(pixel_color.y());
            let b_gamma: f64 = Self::linear_to_gamma(pixel_color.z());

            let intensity = Interval::new(0_f64, 0.999);
            let r: u16 = (u16::MAX as f64 * intensity.clamp(r_gamma)) as u16;
            let g: u16 = (u16::MAX as f64 * intensity.clamp(g_gamma)) as u16;
            let b: u16 = (u16::MAX as f64 * intensity.clamp(b_gamma)) as u16;

            bar.inc(1);
            image::Rgb([r, g, b])
        });
        bar.finish();

        adaptive_img
    }

    fn init(
        defocus_angle: f64,
        focus_dist: f64,
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        image_width: u32,
        samples: u32,
    ) -> Self {
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

        // camera
        // Viewport widths less than one are ok since they are real valued.
        //let lookfrom: Point3 = Point3::new(0_f64, 0_f64, 0_f64);
        //let lookat: Point3 = Point3::new(0_f64, 0_f64, -1_f64);
        //let vup: Vec3 = Vec3::new(0_f64, 1_f64, 0_f64);

        let camera_center = lookfrom;
        //let focal_length = (lookfrom - lookat).length();
        let theta: f64 = degrees_to_radians(vfov);
        let h: f64 = f64::tan(theta / 2_f64);
        let viewport_height = 2_f64 * h * focus_dist;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = Vec3::unit_vector(lookfrom - lookat);
        let u = Vec3::unit_vector(Vec3::cross(&vup, &w));
        let v = Vec3::cross(&w, &u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = viewport_width * u; // Vector across viewport horizontal edge
        let viewport_v = viewport_height * -v; // Vector down viewport vertical edge

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            camera_center - (focus_dist * w) - viewport_u / 2_f64 - viewport_v / 2_f64;
        //let viewport_upper_left = camera_to_viewport_vec - (0.5 * viewport_u) - (0.5 * viewport_v);
        let pixel_00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = focus_dist * (degrees_to_radians(defocus_angle / 2.0)).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        let max_depth: u32 = 100;

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
            max_depth,
            vfov,
            lookfrom,
            lookat,
            vup,
            w,
            u,
            v,
            defocus_angle,
            focus_dist,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    fn get_ray(&self, i: u32, j: u32) -> Ray {
        // Construct a camera ray originating from the defocus disk and directed at a randomly
        // sampled point around the pixel location i, j.

        let offset: Vec3 = Self::sample_square();
        let pixel_sample: Vec3 = self.pixel_00_loc
            + ((i as f64 + offset.x()) * self.pixel_delta_u)
            + ((j as f64 + offset.y()) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction: Vec3 = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn sample_square() -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0_f64)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p: Vec3 = Vec3::random_in_unit_disk();
        self.center + (p[0] * self.defocus_disk_u) + (p[1] * self.defocus_disk_v)
    }

    fn ray_color(r: &Ray, depth: u32, world: &dyn Hittable) -> Color {
        if depth <= 0 {
            return Color::default();
        }

        let mut rec: HitRecord = HitRecord::default();

        if world.hit(&r, &Interval::new(0.001, f64::INFINITY), &mut rec) {
            let mut scattered: Ray = Ray::new(Point3::default(), Vec3::default());
            let mut attenuation: Color = Color::default();

            if rec.mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                return attenuation * Self::ray_color(&scattered, depth - 1, world);
            }
            return Color::default();
        }

        let unit_direction = Vec3::unit_vector(r.direction);
        let a = 0.5 * (unit_direction.y() + 1.0);

        (1.0 - a) * Color::new(1_f64, 1_f64, 1_f64) + a * Color::new(0.5, 0.7, 1.0)
    }
}
