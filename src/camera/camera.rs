use std::collections::HashMap;
use std::u8;

use eframe::egui::{Color32, ColorImage};
use image::{ImageBuffer, Rgb};
use indicatif::ProgressBar;
use micromath::F32Ext;

use crate::geometry::ray::Ray;
use crate::geometry::vec3::{Color, Point3, Vec3};
use crate::geometry::interval::Interval;

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
        samples: u32
    ) -> Self {
        Self::init(defocus_angle, focus_dist, lookfrom, lookat, vup, vfov, aspect_ratio, image_width, samples)
    }

    fn linear_to_gamma(linear_component: f64) -> f64 {
        if linear_component > 0_f64 {
            return f64::sqrt(linear_component);
        }

        0_f64
    }

    fn quantize_color(color: Rgb<u16>, bins: u8) -> Rgb<u16> {
        let c: [f64; 3] = [color[0] as f64, color[1] as f64, color[2] as f64];
        
        image::Rgb([
            (c[0].clamp(0.0, 0.999) * bins as f64) as u16,
            (c[1].clamp(0.0, 0.999) * bins as f64) as u16,
            (c[2].clamp(0.0, 0.999) * bins as f64) as u16
        ])
    }

    fn bits_for_bins<K, V>(hist: &HashMap<K, V>) -> u32 {
        let n = hist.len();
        if n <= 1 {
            // 0 bits if there's 0 or 1 bin: no choice to encode
            return 0;
        }
        // On a 64‑bit platform, usize::BITS == 64.  For 32‑bit it's 32.
        // (n-1).leading_zeros() gives floor_log2(n-1) as:
        //    floor_log2(n-1) = (usize::BITS - 1) - ((n-1).leading_zeros())
        // so subtracting from usize::BITS yields ceil_log2(n).
        usize::BITS - (n - 1).leading_zeros()
    }

    fn pixel_entropy(samples: &[Rgb<u16>], bins: u8, fast_math: bool) -> f64 {
        if samples.is_empty() { return 0.0; }
        let mut histogram = HashMap::new();

        for &sample in samples {
            let q_color = Self::quantize_color(sample, bins);
            *histogram.entry(q_color).or_insert(0u32) += 1;
        }

        let total = samples.len() as f64;
        let log2_total = total.log2();

        histogram.values()
            .map(|&count| {
                let p = count as f64 / total;

                if fast_math {
                    let e = 31 - count.leading_zeros();
                    p * (log2_total - e as f64)
                } else {
                    -p * p.log2()
                }
            })
            .sum()
    }

    fn exp_pixel_entropy(samples: &[Rgb<u16>], bins: u8) -> f32 {
        if samples.is_empty() { return 0.0; }
        let mut histogram = HashMap::new();

        for &sample in samples {
            let q_color = Self::quantize_color(sample, bins);
            *histogram.entry(q_color).or_insert(0u32) += 1;
        }

        let total: f32 = samples.len() as f32;
        let log2_total: f32 = <f32 as F32Ext>::log2(total);

        histogram.values()
            .map(|&count| {
                let p = count as f32 / total;
                let log2_ci = <f32 as F32Ext>::log2(count as f32);
                p * (log2_total - log2_ci)
            })
            .sum()
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

    fn sample_color_u16(ray: &Ray, depth: u32, world: &dyn Hittable) -> Rgb<u16> {
        let c = Self::ray_color(ray, depth, world);
        Rgb([
            (u16::MAX as f64 * c.x().clamp(0.0, 0.999)) as u16,
            (u16::MAX as f64 * c.y().clamp(0.0, 0.999)) as u16,
            (u16::MAX as f64 * c.z().clamp(0.0, 0.999)) as u16,
        ])
    }
    
    fn sample_color(ray: &Ray, depth: u32, world: &dyn Hittable) -> [f64; 3] {
        let c = Self::ray_color(ray, depth, world);
        [c.x(), c.y(), c.z()]
    }

    pub fn entropy_adaptive_render(
        &self,
        world: &dyn Hittable,
        bins: u8,
        min_samples: u32,
        max_samples: u32,
        gamma: f64,
        entropy_threshold: f64,
        fast_math: bool,
    ) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let (width, height) = (self.image_width, self.image_height);
        let max_entropy = (bins as u32).pow(3) as f64; // bins^3 possibilities
        let max_entropy_bits = max_entropy.log2();     // max bits of entropy

        let bar = ProgressBar::new((width * height) as u64);

        ImageBuffer::from_fn(width, height, |i, j| {
            // STEP 1: Initial 16-sample entropy estimate
            let mut initial_samples = vec![];
            for _ in 0..16 {
                let ray = self.get_ray(i, j);
                let color = Self::sample_color_u16(&ray, self.max_depth, world);
                initial_samples.push(color);
            }

            let (h1, h2) = if fast_math {
                let mid = 8;
                let h1 = Self::exp_pixel_entropy(&initial_samples[..mid], bins) as f64;
                let h2 = Self::exp_pixel_entropy(&initial_samples[mid..], bins) as f64;
                (h1, h2)
            } else {
                let mid = 8;
                let h1 = Self::pixel_entropy(&initial_samples[..mid], bins, false);
                let h2 = Self::pixel_entropy(&initial_samples[mid..], bins, false);
                (h1, h2)
            };

            let mut entropy_est = h2;
            if (h2 - h1).abs() > entropy_threshold {
                // Not stable: take 16 more samples to refine
                for _ in 0..16 {
                    let ray = self.get_ray(i, j);
                    let color = Self::sample_color_u16(&ray, self.max_depth, world);
                    initial_samples.push(color);
                }

                entropy_est = if fast_math {
                    Self::exp_pixel_entropy(&initial_samples, bins) as f64
                } else {
                    Self::pixel_entropy(&initial_samples, bins, false)
                };
            }

            // STEP 2: Compute adaptive sample count from entropy
            let normalized_entropy = (entropy_est / max_entropy_bits).clamp(0.0, 1.0);
            let adaptive_samples = min_samples + ((max_samples - min_samples) as f64 * normalized_entropy.powf(gamma)) as u32;

            // STEP 3: Final sampling pass
            let mut accum = [0.0; 3];
            for _ in 0..adaptive_samples {
                let ray = self.get_ray(i, j);
                let color = Self::sample_color(&ray, self.max_depth, world);
                for k in 0..3 {
                    accum[k] += color[k];
                }
            }
            for k in 0..3 {
                accum[k] /= adaptive_samples as f64;
            }

            let to_u16 = |v: f64| -> u16 {
                let gamma_corrected = v.sqrt(); // gamma 2.0
                (u16::MAX as f64 * gamma_corrected.clamp(0.0, 0.999)) as u16
            };

            bar.inc(1);
            Rgb([
                to_u16(accum[0]),
                to_u16(accum[1]),
                to_u16(accum[2]),
            ])
        })
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

    fn init(
        defocus_angle: f64,
        focus_dist: f64,
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64, 
        aspect_ratio: f64, 
        image_width: u32, 
        samples: u32
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
        let viewport_upper_left = camera_center - (focus_dist * w) 
            - viewport_u / 2_f64 - viewport_v / 2_f64;
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
