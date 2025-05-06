use crate::geometry::vec3::Vec3;
use crate::hittables::cpu::sphere::Sphere;
use crate::HittableList;
use crate::material::kind::MatKind;

use bytemuck::{Pod, Zeroable};
use rustacuda::memory::DeviceBuffer;

pub const MAX_MAT_PARAMS: usize = 4;

// C-ABI-safe inline sphere primitive.
#[repr(C)]
#[derive(DeviceCopy, Copy, Clone, Pod, Zeroable, Debug)]
pub struct SpherePrimitive {
    pub center: [f32; 3], // maps to float3
    pub radius: f32,
    pub mat_idx: u32, // index into material buffer
}

// C-ABI-safe inline material primitive (fixed-size parameter storage).
#[repr(C)]
#[derive(DeviceCopy, Copy, Clone, Pod, Zeroable, Debug)]
pub struct MaterialPrimitive {
    pub kind: u32,                     // material type tag ( header )
    pub param_count: u32,              // how many slots in `params` are valid ( header )
    // if header size exceeds 16 bytes need to rethink your padding to keep that header a 
    // nice power-of-two or multiple of the cache line size.
    pub _pad: [u32; 2],                
    pub params: [f32; MAX_MAT_PARAMS], // inline parameters
}

#[derive(Debug)]
pub struct GPUScene {
    pub spheres: DeviceBuffer<SpherePrimitive>,
    pub materials: DeviceBuffer<MaterialPrimitive>,
}

impl GPUScene {
    fn marshal_scene(world: &HittableList) -> (Vec<SpherePrimitive>, Vec<MaterialPrimitive>) {
        let mut spheres: Vec<SpherePrimitive> = Vec::new();
        let mut materials: Vec<MaterialPrimitive> = Vec::new();

        for (_obj_idx, object) in world.into_iter().enumerate() {
            if let Some(sphere) = object.downcast_ref::<Sphere>() {
                let c: Vec3 = sphere.center;
                let center = [c.x(), c.y(), c.z()];
                let mat_idx = materials.len() as u32;

                spheres.push(SpherePrimitive {
                    center,
                    radius: sphere.radius,
                    mat_idx,
                });

                // Build material primitive from its trait object
                let mat_ref = sphere.mat.as_ref();
                let (kind, raw_params) = 
                    match MatKind::try_from(mat_ref).unwrap() { // panics if unsupported material
                        MatKind::Lambertian(l) => {
                            let c = l.albedo();
                            (0, vec![c.x(), c.y(), c.z()])
                        }
                        MatKind::Metal(m) => {
                            let c = m.albedo();
                            (1, vec![c.x(), c.y(), c.z(), m.fuzz()])
                        }
                        MatKind::Dielectric(d) => {
                            (2, vec![d.refraction_index()])
                        }
                    };

                // Inline into fixed-size array
                let mut params = [0.0f32; MAX_MAT_PARAMS];
                params[..raw_params.len()].copy_from_slice(&raw_params);

                materials.push(MaterialPrimitive {
                    kind,
                    param_count: raw_params.len() as u32,
                    _pad: [0; 2],
                    params,
                });
            }
        }

        (spheres, materials)
    }
}

impl TryFrom<&HittableList> for GPUScene {
    type Error = rustacuda::error::CudaError;

    fn try_from(world: &HittableList) -> Result<Self, Self::Error> {
        let (sphere_prims, mat_prims) = GPUScene::marshal_scene(world);
        let sphere_buf = DeviceBuffer::from_slice(
            bytemuck::cast_slice(&sphere_prims))?;
        let mat_buf = DeviceBuffer::from_slice(
            bytemuck::cast_slice(&mat_prims))?;
        
        Ok(GPUScene {
            spheres: sphere_buf,
            materials: mat_buf,
        })
    }
}
