# Rust Ray Tracer

A personal graphics programming project in Rust, inspired by Peter Shirley’s **Ray Tracing in One Weekend**.  
Serves as both a learning exercise and a foundation for experimenting with rendering techniques and graphics systems from first principles.

---

## Table of Contents

1. [Overview](#overview)  
2. [Key Features](#key-features)  
3. [Repository Layout](#repository-layout)  
4. [Rendering Modes](#rendering-modes)  
   - [Static Render to PNG](#static-render-to-png)  
   - [Interactive egui Viewer](#interactive-egui-viewer)  
5. [Goals & Next Steps](#goals--next-steps)  
6. [Philosophy](#philosophy)  

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
- Progressive rendering with an optional `egui` frontend  

---

## Key Features

- **Book-accurate architecture**  
  Follows the structure of *Ray Tracing in One Weekend* in Rust.

- **Modular scene description**  
  Uses a `Hittable` trait with dynamic dispatch for flexible scene composition.

- **Material abstraction**  
  Clean separation of material behaviors via the `Material` trait.

- **Multisampling camera**  
  Customizable resolution, sample count, and field-of-view.

- **Progressive rendering interface**  
  Double-buffered image updates using `eframe` + `egui` for interactive previews.

- **Lock-free double buffering**  
  Atomic front-buffer swapping for safe, efficient frame updates.

---

## Repository Layout

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

> This project is not just about rendering pretty pictures—it's about understanding the math and data structures that underpin modern graphics.
