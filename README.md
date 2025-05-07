# Rust Ray Tracer 🦀🌟

A personal graphics programming project in Rust, inspired by Peter Shirley’s **Ray Tracing in One Weekend**.  
Serves as both a learning exercise and a foundation for experimenting with rendering techniques and graphics systems from first principles. 🚀

---

## Overview

Implements a **CPU-based ray tracer** from scratch in safe, idiomatic Rust.  
Supported features:

- Ray–sphere intersections  
- Lambertian (diffuse) and Metal (reflective) materials  
- Dielectric (glass-like) materials  
- Shadow rays and surface normals  
- Gamma correction  
- Anti-aliasing via stochastic sampling  
- Defocus blur (depth of field)    
---

## Key Features

- **Modular scene description**
  Uses a `Hittable` trait with dynamic dispatch for flexible scene composition.

- **Material abstraction** 
  Clean separation of material behaviors via the `Material` trait.

- **Multisampling camera**
  Customizable resolution, sample count, and field-of-view.
---

## Repository Layout 📂

```text
├── camera.rs               # Ray generation, sampling, DoF, and rendering logic
├── hittable.rs             # Trait + hit record for intersection handling
├── hittable_list.rs        # Aggregates multiple hittables with closest-hit selection
├── sphere.rs               # Basic sphere object implementing Hittable
├── material/
│   ├── lambertian.rs       # Diffuse reflection model
│   ├── metal.rs            # Reflective material
│   ├── dielectric.rs       # Glass-like refraction model
│   └── material.rs         # Trait and dynamic material system
├── geometry/
│   ├── vec3.rs             # Core vector math
│   ├── ray.rs              # Ray representation
│   └── interval.rs         # Numerical range utility
├── double_buffer.rs        # Lock-free reader/writer pattern for progressive render
├── command.rs              # Render control commands/events
├── app.rs                  # egui front-end
└── main.rs                 # CLI and static image renderer
````
## Goals & Next Steps 🎯
* Add BVH acceleration structure
* Add textures (checkered, image-based)
* Add lights 💡 and emissive materials
* Support volumetrics
* GPU acceleration (via `wgpu`, CUDA, or Metal) ([cuda_tracer branch](https://github.com/Anjin-Byte/shiny-parakeet/tree/cuda_tracer))
* Explore adaptive sampling and denoising ([entropy informed adaptive](https://github.com/Anjin-Byte/shiny-parakeet/tree/entropy_adaptive))
  ([variance informed adaptive](https://github.com/Anjin-Byte/shiny-parakeet/tree/adaptive_sampling))

---
> This project is not just about rendering pretty pictures—it's about understanding the math and data structures that underpin modern graphics.
> But, yes, I would love to render some pretty pictures along the way :)
