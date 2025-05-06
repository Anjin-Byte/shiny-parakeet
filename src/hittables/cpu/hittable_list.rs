use crate::geometry::interval::Interval;
use crate::geometry::ray::Ray;

use super::hittable::HitRecord;
use super::hittable::Hittable;
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let mut hit_anything: bool = false;
        let mut closest_so_far: f32 = ray_t.max;
        

        for object in self.objects.iter() {
            let mut temp_rec: HitRecord = HitRecord::default();
            if object.hit(&r, &Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec;
            }
        }

        hit_anything
    }
}


impl<'a> IntoIterator for &'a HittableList {
    type Item = &'a dyn Hittable;

    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, Box<dyn Hittable>>,
        fn(&Box<dyn Hittable>) -> &dyn Hittable
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.objects.iter().map(|b| b.as_ref())
    }
}
