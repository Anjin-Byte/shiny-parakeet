use std::sync::Arc;

use crate::geometry::ray::Ray;
use crate::geometry::interval::Interval;

use super::hittable::Hittable;
use super::hittable::HitRecord;

pub struct HittableList {
    objects: Vec<Arc<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(
        &self, 
        r: &Ray, 
        ray_t: &Interval, 
        rec: &mut HitRecord
    ) -> bool {
        let mut temp_rec: HitRecord = HitRecord::default();
        let mut hit_anything: bool = false;
        let mut closest_so_far: f64 = ray_t.max;
        
        for object in self.objects.iter() {
            if object.hit(&r, &Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec;
            }
        }

        hit_anything
    }
}
