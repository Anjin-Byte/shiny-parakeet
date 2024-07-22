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
