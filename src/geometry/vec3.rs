use std::ops;

use crate::{random_double, random_double_range};

pub type Point3 = Vec3;
pub type Color = Vec3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec3 {
    pub e: [f64; 3],
}

impl Vec3 {
    pub fn default() -> Self {
        Self {
            e: [0_f64, 0_f64, 0_f64]
        }
    }

    pub fn new(e0: f64, e1: f64, e2:f64) -> Self {
        Self {
            e: [e0, e1, e2]
        }
    }

    pub fn random() -> Self {
        Self {
            e: [
                random_double(),
                random_double(),
                random_double()
            ],
        }
    }

    pub fn random_range(min: f64, max: f64) -> Self {
        Self {
            e: [
                random_double_range(min, max),
                random_double_range(min, max),
                random_double_range(min, max)
            ],
        }
    }

    fn random_in_unit_sphere() -> Vec3 {
        loop {
            let p = Self::random_range(-1_f64, 1_f64);
            if p.length_squared() < 1_f64 { 
                return p; 
            }
        }
    }

    pub fn random_on_hemisphere(normal: &Vec3) -> Self {
        let on_unit_sphere: Vec3 = Self::random_unit_vector();
        if Self::dot(&on_unit_sphere, normal) > 0_f64 {
            return on_unit_sphere;
        } else {
            return -1_f64 * on_unit_sphere;
        }
    }

    pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
        *v - 2_f64 * Self::dot(v, n) * *n
    }

    pub fn refract(uv: &Vec3, normal: &Vec3, eta_over_eta_prime: f64) -> Vec3 {
        let cos_theta: f64 = Self::dot(&-uv, normal).min(1_f64);
        let r_out_perp: Vec3 = eta_over_eta_prime * (*uv + cos_theta * *normal);

        let len_sq = r_out_perp.length_squared();
        let under_sqrt = 1.0 - len_sq;
        let r_out_parallel = if under_sqrt >= 0.0 {
            -under_sqrt.sqrt() * *normal
        } else {
            // total internal reflection or numerical error: fallback to zero vector
            // this is to address potential NaN return from f64::sqrt()
            Vec3::default()
        };

        r_out_parallel + r_out_perp // ∥rout​∥2=∥rout⊥​∥2+∥rout∥​∥2
    }

    pub fn random_unit_vector() -> Vec3 {
        Self::unit_vector(Self::random_in_unit_sphere())
    }

    pub fn x(&self) -> f64 {
        self.e[0]
    }

    pub fn y(&self) -> f64 {
        self.e[1]
    }

    pub fn z(&self) -> f64 {
        self.e[2]
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.e[0] * self.e[0] + 
        self.e[1] * self.e[1] + 
        self.e[2] * self.e[2]
    } 

    pub fn near_zero(&self) -> bool {
        let s: f64 = 1e-8;
        (f64::abs(self.e[0]) < s) && (f64::abs(self.e[1]) < s) && (f64::abs(self.e[2]) < s)
    }

    pub fn dot(lhs: &Vec3, rhs: &Vec3) -> f64 {
        lhs.e[0] * rhs.e[0] +
        lhs.e[1] * rhs.e[1] +
        lhs.e[2] * rhs.e[2]
    } 

    pub fn cross(lhs: &Vec3, rhs: &Vec3) -> Vec3 {
        Vec3::new(
            lhs.e[1] * rhs.e[2] - lhs.e[2] * rhs.e[1],
            lhs.e[2] * rhs.e[0] - lhs.e[0] * rhs.e[2],
            lhs.e[0] * rhs.e[1] - lhs.e[1] * rhs.e[0],
        )    
    }

    pub fn unit_vector(v: Vec3) -> Vec3 {
        (1_f64 / v.length()) * v
    }
}

impl<'a> ops::Neg for &'a Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::new(-self.e[0], -self.e[1], -self.e[2])
    }
}

impl ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3::new(-self.e[0], -self.e[1], -self.e[2])
    }
}

impl ops::Index<usize> for  Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.e[index]
    }
}

impl ops::IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.e[index]
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.e[0] + rhs.e[0], 
            self.e[1] + rhs.e[1], 
            self.e[2] + rhs.e[2]
        )
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = Vec3::new(
            self.e[0] + rhs.e[0], 
            self.e[1] + rhs.e[1], 
            self.e[2] + rhs.e[2]
        );
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.e[0] - rhs.e[0], 
            self.e[1] - rhs.e[1], 
            self.e[2] - rhs.e[2]
        )
    }
}

impl ops::SubAssign<Vec3> for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = Vec3::new(
            self.e[0] - rhs.e[0], 
            self.e[1] - rhs.e[1], 
            self.e[2] - rhs.e[2]
        );
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.e[0] * rhs.e[0], 
            self.e[1] * rhs.e[1], 
            self.e[2] * rhs.e[2]
        )
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(
            self.e[0] * rhs, 
            self.e[1] * rhs, 
            self.e[2] * rhs
        )
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self * rhs.e[0], 
            self * rhs.e[1], 
            self * rhs.e[2]
        )
    }
}

impl ops::MulAssign<Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        *self = Vec3::new(
            self.e[0] * rhs.e[0], 
            self.e[1] * rhs.e[1], 
            self.e[2] * rhs.e[2]
        );
    }
}

impl ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = Vec3::new(
            self.e[0] * rhs, 
            self.e[1] * rhs, 
            self.e[2] * rhs
        );
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        (1_f64 / rhs) * self
    }
}
