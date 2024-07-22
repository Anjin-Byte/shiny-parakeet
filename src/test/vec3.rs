#[allow(unused_imports)]
use crate::geometry::vec3::Vec3;

#[test]
fn test_default() {
    let v = Vec3::default();
    assert_eq!(v.e, [0.0, 0.0, 0.0]);
}

#[test]
fn test_new() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v.e, [1.0, 2.0, 3.0]);
}

#[test]
fn test_x() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v.x(), 1.0);
}

#[test]
fn test_y() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v.y(), 2.0);
}

#[test]
fn test_z() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v.z(), 3.0);
}

#[test]
fn test_length() {
    let v = Vec3::new(3.0, 4.0, 0.0);
    assert_eq!(v.length(), 5.0);

    let u = Vec3::new(1.0, 2.0, 2.0);
    assert!((u.length() - 3.0).abs() < 1e-10);
}

#[test]
fn test_length_squared() {
    let v = Vec3::new(3.0, 4.0, 0.0);
    assert_eq!(v.length_squared(), 25.0);

    let u = Vec3::new(1.0, 2.0, 2.0);
    assert!((u.length_squared() - 9.0).abs() < 1e-10);
}

#[test]
fn test_dot() {
    // Orthogonal vectors
    let v = Vec3::new(1.0, 0.0, 0.0);
    let u = Vec3::new(0.0, 1.0, 0.0);
    assert_eq!(Vec3::dot(&v, &u), 0.0, "Dot product of orthogonal vectors is not zero");

    // Parallel vectors
    let v = Vec3::new(1.0, 2.0, 3.0);
    let u = Vec3::new(2.0, 4.0, 6.0);
    assert_eq!(Vec3::dot(&v, &u), 28.0, "Dot product of parallel vectors is incorrect");

    // Anti-parallel vectors
    let v = Vec3::new(1.0, 2.0, 3.0);
    let u = Vec3::new(-1.0, -2.0, -3.0);
    assert_eq!(Vec3::dot(&v, &u), -14.0, "Dot product of anti-parallel vectors is incorrect");

    // Vectors with zero components
    let v = Vec3::new(1.0, 0.0, 0.0);
    let u = Vec3::new(0.0, 2.0, 0.0);
    assert_eq!(Vec3::dot(&v, &u), 0.0, "Dot product with zero components is incorrect");

    // Vectors with very large values
    let v = Vec3::new(1e10, 2e10, 3e10);
    let u = Vec3::new(1e10, -2e10, 3e10);
    assert_eq!(Vec3::dot(&v, &u), 6e20, "Dot product of large vectors is incorrect");

    // Vectors with very small values
    //let v = Vec3::new(1e-10, 2e-10, 3e-10);
    //let u = Vec3::new(1e-10, -2e-10, 3e-10);
    //assert_eq!(v.dot(u), 6e-20, "Dot product of small vectors is incorrect");
}

#[test]
fn test_cross() {
    // Orthogonal vectors
    let v = Vec3::new(1.0, 0.0, 0.0);
    let u = Vec3::new(0.0, 1.0, 0.0);
    let cross_product = Vec3::cross(&v, &u);
    assert_eq!(cross_product.e, [0.0, 0.0, 1.0], "Cross product of orthogonal vectors is incorrect");

    // Parallel vectors
    let v = Vec3::new(1.0, 2.0, 3.0);
    let u = Vec3::new(2.0, 4.0, 6.0);
    let cross_product = Vec3::cross(&v, &u);
    assert_eq!(cross_product.e, [0.0, 0.0, 0.0], "Cross product of parallel vectors is not zero vector");

    // Anti-parallel vectors
    let v = Vec3::new(1.0, 2.0, 3.0);
    let u = Vec3::new(-1.0, -2.0, -3.0);
    let cross_product = Vec3::cross(&v, &u);
    assert_eq!(cross_product.e, [0.0, 0.0, 0.0], "Cross product of anti-parallel vectors is not zero vector");

    // Vectors with zero components
    let v = Vec3::new(1.0, 0.0, 0.0);
    let u = Vec3::new(0.0, 0.0, 1.0);
    let cross_product = Vec3::cross(&v, &u);
    assert_eq!(cross_product.e, [0.0, -1.0, 0.0], "Cross product with zero components is incorrect");

    // Vectors with very large values
    let v = Vec3::new(1e10, 2e10, 3e10);
    let u = Vec3::new(1e10, -2e10, 3e10);
    let cross_product = Vec3::cross(&v, &u);
    assert_eq!(cross_product.e, [1.2e21, 0.0, -4e20], "Cross product of large vectors is incorrect");

    // Vectors with very small values
    //let v = Vec3::new(1e-10, 2e-10, 3e-10);
    //let u = Vec3::new(1e-10, -2e-10, 3e-10);
    //let cross_product = v.cross(u);
    //assert_eq!(cross_product.e, [1.2e-19, 0.0, -4e-20], "Cross product of small vectors is incorrect");
}

#[test]
fn test_unit_vector() {
    let v = Vec3::new(3.0, 4.0, 0.0);
    let unit_v = Vec3::unit_vector(v);
    assert!((unit_v.length() - 1.0).abs() < 1e-10, "Length of unit vector is not 1");
    assert!((unit_v.x() - 0.6).abs() < 1e-10, "Unit vector x component incorrect");
    assert!((unit_v.y() - 0.8).abs() < 1e-10, "Unit vector y component incorrect");
    assert!((unit_v.z() - 0.0).abs() < 1e-10, "Unit vector z component incorrect");

    /*     
    // Test another vector
    let u = Vec3::new(1.0, 2.0, 2.0);
    let unit_u = Vec3::unit_vector(u);
    assert!((unit_u.length() - 1.0).abs() < 1e-10, "Length of unit vector is not 1");
    assert!((unit_u.x() - (1_f64 / 3_f64).sqrt()).abs() < 1e-10, "Unit vector x component incorrect");
    assert!((unit_u.y() - (2_f64 / 3_f64).sqrt()).abs() < 1e-10, "Unit vector y component incorrect");
    assert!((unit_u.z() - (2_f64 / 3_f64).sqrt()).abs() < 1e-10, "Unit vector z component incorrect"); 
    */

    // Test unit vector of a zero vector
    let zero_v = Vec3::new(0.0, 0.0, 0.0);
    let unit_zero_v = Vec3::unit_vector(zero_v);
    assert!(unit_zero_v.e.iter().all(|&x| x.is_nan()), "Unit vector of zero vector should contain NaNs");

    // Test very small vector
    let small_v = Vec3::new(1e-10, 1e-10, 1e-10);
    let unit_small_v = Vec3::unit_vector(small_v);
    assert!((unit_small_v.length() - 1.0).abs() < 1e-10, "Length of unit vector is not 1 for small vector");

    // Test very large vector
    let large_v = Vec3::new(1e10, 1e10, 1e10);
    let unit_large_v = Vec3::unit_vector(large_v);
    assert!((unit_large_v.length() - 1.0).abs() < 1e-10, "Length of unit vector is not 1 for large vector");
}

#[test]
fn test_index() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v[0], 1.0);
    assert_eq!(v[1], 2.0);
    assert_eq!(v[2], 3.0);
}

#[test]
fn test_index_mut() {
    let mut v = Vec3::new(1.0, 2.0, 3.0);
    v[0] = 4.0;
    v[1] = 5.0;
    v[2] = 6.0;
    assert_eq!(v[0], 4.0);
    assert_eq!(v[1], 5.0);
    assert_eq!(v[2], 6.0);
}

#[test]
#[should_panic]
fn test_index_out_of_bounds() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    let _ = v[3]; // This should panic
}

#[test]
#[should_panic]
fn test_index_mut_out_of_bounds() {
    let mut v = Vec3::new(1.0, 2.0, 3.0);
    v[3] = 4.0; // This should panic
}

#[test]
fn test_index_range() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    for i in 0..3 {
        assert_eq!(v[i], v.e[i]);
    }
}

#[test]
fn test_index_mut_range() {
    let mut v = Vec3::new(1.0, 2.0, 3.0);
    for i in 0..3 {
        v[i] = v[i] + 1.0;
    }
    assert_eq!(v[0], 2.0);
    assert_eq!(v[1], 3.0);
    assert_eq!(v[2], 4.0);
}

#[test]
fn test_mul_f64_vec3() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    let scaled_v = 2.0 * v;
    assert_eq!(scaled_v.e, [2.0, 4.0, 6.0]);
}

#[test]
fn test_mul_vec3_f64() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    let scaled_v = v * 2.0;
    assert_eq!(scaled_v.e, [2.0, 4.0, 6.0]);
}

#[test]
fn test_mul_zero() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    let scaled_v1 = 0.0 * v;
    let scaled_v2 = v * 0.0;
    assert_eq!(scaled_v1.e, [0.0, 0.0, 0.0]);
    assert_eq!(scaled_v2.e, [0.0, 0.0, 0.0]);
}

#[test]
fn test_mul_negative() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    let scaled_v1 = -1.0 * v;
    let scaled_v2 = v * -1.0;
    assert_eq!(scaled_v1.e, [-1.0, -2.0, -3.0]);
    assert_eq!(scaled_v2.e, [-1.0, -2.0, -3.0]);
}

#[test]
fn test_mul_fraction() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    let scaled_v1 = 0.5 * v;
    let scaled_v2 = v * 0.5;
    assert_eq!(scaled_v1.e, [0.5, 1.0, 1.5]);
    assert_eq!(scaled_v2.e, [0.5, 1.0, 1.5]);
}