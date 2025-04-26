use std::collections::HashMap;
use std::u8;

use eframe::egui::{Color32, ColorImage};
use image::imageops::{blur, resize, FilterType};
use image::{imageops, ImageBuffer, Luma, Rgb};
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

        let entropy = histogram.values()
            .map(|&count| {
                let p = count as f64 / total;

                if fast_math {
                    let e = 31 - count.leading_zeros();
                    p * (log2_total - e as f64)
                } else {
                    println!("entropy: {}", -p * p.log2());
                    -p * p.log2()
                }
            })
            .sum();

        println!("entropy: {}", entropy);
        entropy
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

    pub fn pre_compute_adaptive_render(
        &self,
        world: &dyn Hittable,
        luma_map: &ImageBuffer<Luma<u16>, Vec<u16>>,
        min_samples: u32,
        max_samples: u32,
        gamma: f64,
    ) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        assert_eq!(
            (self.image_width, self.image_height),
            luma_map.dimensions(),
            "Camera resolution and entropy map must match"
        );

        println!("Rendering(adaptive) image...");
        let total = (self.image_width as u64) * (self.image_height as u64);
        let bar   = indicatif::ProgressBar::new(total);

        ImageBuffer::from_fn(self.image_width, self.image_height, |i, j| {
            let luma: u16 = luma_map.get_pixel(i, j)[0];
            let normalized_entropy = (luma as f64) / (u16::MAX as f64);

            let adaptive_samples = min_samples
                + ((max_samples - min_samples) as f64 * normalized_entropy.powf(gamma)) as u32;

            let mut accum = [0.0; 3];
            for _ in 0..adaptive_samples {
                let ray   = self.get_ray(i, j);
                let color = Self::sample_color(&ray, self.max_depth, world);
                for k in 0..3 {
                    accum[k] += color[k];
                }
            }
            for k in 0..3 {
                accum[k] /= adaptive_samples as f64;
            }

            let to_u16 = |v: f64| -> u16 {
                let gamma_corrected = v.sqrt();
                (u16::MAX as f64 * gamma_corrected.clamp(0.0, 0.999)) as u16
            };

            bar.inc(1);
            Rgb([to_u16(accum[0]), to_u16(accum[1]), to_u16(accum[2])])
        })
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

            let normalized_entropy = (entropy_est / max_entropy_bits).clamp(0.0, 1.0);
            let adaptive_samples = min_samples + ((max_samples - min_samples) as f64 * normalized_entropy.powf(gamma)) as u32;

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

    /// Render a heatmap of per‑pixel entropy over the scene.
    ///
    /// For each pixel:
    /// 1. Take an initial batch of 16 samples, compute H₁ and H₂ on halves.
    /// 2. If |H₂−H₁| > `entropy_threshold`, take 16 more samples and recompute.
    /// 3. Normalize entropy by `max_entropy_bits = log2(bins³)`.
    /// 4. Map normalized entropy ∈ [0,1] to a blue→red gradient.
    ///
    /// # Parameters
    /// - `world` – scene to trace against
    /// - `bins` – number of quantization bins per channel
    /// - _unused_ `min_samples`, `max_samples`, `gamma` – only entropy matters here
    /// - `entropy_threshold` – when to refine entropy estimate
    /// - `fast_math` – whether to use the `exp_pixel_entropy` shortcut
    ///
    /// # Returns
    /// A heatmap image where blue = low entropy, red = high entropy.
    pub fn entropy_heatmap(
        &self,
        world: &dyn Hittable,
        bins: u32,
        entropy_threshold: f64,
    ) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let (w, h) = (self.image_width, self.image_height);
        let max_bits = (bins.pow(3) as f64).log2();
        let bar = ProgressBar::new((w * h) as u64);

        imageops::fast_blur(&ImageBuffer::from_fn(w, h, |i, j| {
            let mut samples = Vec::with_capacity(128);
            for _ in 0..64 {
                let ray = self.get_ray(i, j);
                let col = Self::ray_color(&ray, self.max_depth, world);
                samples.push(col);
            }

            // helper: quantize a Vec3 ∈ [0,1]^3 into (r_i,g_i,b_i) in 0..bins-1
            fn quantize(c: Vec3, bins: u32) -> (u32,u32,u32) {
                let f = |x: f64| -> u32 {
                    let t = (x.clamp(0.0,1.0) * (bins as f64 - 1.0)).floor();
                    t as u32
                };
                (f(c.x()), f(c.y()), f(c.z()))
            }

            fn shannon_entropy(slice: &[Vec3], bins: u32) -> f64 {
                let mut hist = HashMap::new();
                for &c in slice {
                    *hist.entry(quantize(c, bins)).or_insert(0u32) += 1;
                }
                let total = slice.len() as f64;
                hist.values()
                    .map(|&cnt| {
                        let p = cnt as f64 / total;
                        -p * p.log2()
                    })
                    .sum()
            }

            let mid = samples.len() / 2;
            let h1 = shannon_entropy(&samples[..mid], bins);
            let h2 = shannon_entropy(&samples[mid..], bins);
            let mut h = h2;
            if (h2 - h1).abs() > entropy_threshold {
                for _ in 0..64 {
                    let ray = self.get_ray(i, j);
                    let col = Self::ray_color(&ray, self.max_depth, world);
                    samples.push(col);
                }
                h = shannon_entropy(&samples, bins);
            }

            let norm = (h / max_bits).clamp(0.0, 1.0);
            // distance from 0.5 in [0,0.5]
            let d = (norm - 0.5).abs();
            // map so that d = 0 → white, d = 0.5 → black
            let intensity = 1.0 - (d * 2.0).clamp(0.0, 1.0);
            let gray_u16 = (intensity * u16::MAX as f64).round() as u16;

            bar.inc(1);
            Rgb([gray_u16, gray_u16, gray_u16])
        }), 3_f32)
    }

    /// Render a temporally‑averaged entropy heatmap.
    ///
    /// For each pixel (i,j), this performs `runs` independent entropy
    /// estimates and averages them to reduce Monte Carlo noise. Each estimate:
    /// 1. Samples 16 rays → colors via `ray_color`
    /// 2. Splits into two halves, computes Shannon entropy on each
    /// 3. If |H₂–H₁| > `entropy_threshold`, takes 16 more samples and recomputes
    /// 4. Returns the Shannon entropy in bits
    ///
    /// After averaging over `runs`, the mean entropy is normalized by
    /// `log₂(bins³)` to [0,1] and mapped to a 16‑bit grayscale.
    ///
    /// # Parameters
    /// - `world`              – the scene to trace
    /// - `bins`               – quantization bins per channel
    /// - `entropy_threshold`  – stability threshold for two‑half check
    /// - `runs`               – number of independent entropy estimations
    ///
    /// # Returns
    /// An `ImageBuffer<Rgb<u16>>` where 0=black (low entropy) and 65535=white (high entropy).
    pub fn entropy_heatmap_temporal(
        &self,
        world: &dyn Hittable,
        bins: u32,
        entropy_threshold: f32,
        runs: u32,
    ) -> ImageBuffer<Luma<u16>, Vec<u16>> {
        let (width, height) = (self.image_width, self.image_height);
        // maximum possible entropy in bits = log2(bins³)
        let max_entropy_bits = (bins.pow(3) as f32).log2();
        let bar = ProgressBar::new((width * height) as u64);

        fn quantize(c: Vec3, bins: u32) -> (u32, u32, u32) {
            let f = |x: f32| ((x.clamp(0.0, 1.0) * (bins as f32 - 1.0)).floor()) as u32;
            (f(c.x() as f32), f(c.y() as f32), f(c.z() as f32))
        }

        fn shannon_entropy(samples: &[Vec3], bins: u32) -> f32 {
            let mut hist = HashMap::new();
            for &col in samples {
                *hist.entry(quantize(col, bins)).or_insert(0u32) += 1;
            }

            let total = samples.len() as f32;
            let log2_total: f32 = <f32 as F32Ext>::log2(total);

            hist.values()
                .map(|&count| {
                    let p = count as f32 / total;
                    let e = 31 - count.leading_zeros();
                    p * (log2_total - e as f32)
                })
                .sum()
        }

        let heatmap = ImageBuffer::from_fn(width, height, |i, j| {
            let mut sum_entropy = 0.0;
            for _ in 0..runs {
                let mut samples = Vec::with_capacity(32);
                for _ in 0..16 {
                    let ray = self.get_ray(i, j);
                    samples.push(Self::ray_color(&ray, self.max_depth, world));
                }

                let mid = samples.len() / 2;
                let h1 = shannon_entropy(&samples[..mid], bins);
                let h2 = shannon_entropy(&samples[mid..], bins);

                let mut h = h2;
                if (h2 - h1).abs() > entropy_threshold {
                    for _ in 0..16 {
                        let ray = self.get_ray(i, j);
                        samples.push(Self::ray_color(&ray, self.max_depth, world));
                    }
                    h = shannon_entropy(&samples, bins);
                }

                sum_entropy += h;
            }

            let avg = sum_entropy / runs as f32;
            let norm = (avg / max_entropy_bits).clamp(0.0, 1.0);
            let intensity = (norm * u16::MAX as f32).round() as u16;

            bar.inc(1);
            Luma([intensity])
        });

        let nwidth = width;
        let nheight = height;

        let sigma = 0.8;
        let blur = imageops::fast_blur(&heatmap, sigma);
        

        let filter = FilterType::Triangle;
        let heatmap_upscale = resize(&blur, nwidth, nheight, filter);


        heatmap_upscale
        
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
        let total_pixels = (self.image_width as u64) * (self.image_height as u64);
        let bar = ProgressBar::new(total_pixels);

        let img = ImageBuffer::from_fn(self.image_width, self.image_height, |i, j| {
            let linear_color = self.sample_pixel_color(i, j, world);
            let pixel = Self::to_rgb16(linear_color);

            bar.inc(1);
            pixel
        });

        bar.finish();
        img
    }

    fn sample_pixel_color(
        &self, 
        i: u32, j: u32, 
        world: &dyn Hittable
    ) -> Color {
        let mut accum = Color::default();
        for _ in 0..self.samples_per_pixel {
            let ray: Ray = self.get_ray(i, j);
            accum += Self::ray_color(&ray, self.max_depth, world);
        }
        accum * self.pixel_samples_scale
    }

    fn to_rgb16(linear: Color) -> Rgb<u16> {
        let r_gamma = Self::linear_to_gamma(linear.x());
        let g_gamma = Self::linear_to_gamma(linear.y());
        let b_gamma = Self::linear_to_gamma(linear.z());

        let clamp = |v: f64| {
            let v = Interval::new(0.0, 0.999).clamp(v);
            (v * (u16::MAX as f64)) as u16
        };

        Rgb([ clamp(r_gamma), clamp(g_gamma), clamp(b_gamma) ])
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

        let adptv_heur_width = image_width / 4;
        let adptv_heur_height = image_height / 4;

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
