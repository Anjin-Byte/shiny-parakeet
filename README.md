Rust Ray Tracer (Based on Ray Tracing in One Weekend)

This is a personal graphics programming project written in Rust, inspired by Peter Shirley’s Ray Tracing in One Weekend. It serves as both a learning exercise and a foundation for experimenting with rendering techniques and graphics systems from first principles.

Overview

The project implements a CPU-based ray tracer from scratch in idiomatic Rust. It currently supports:
	•	Ray-sphere intersection
	•	Lambertian (diffuse) and Metal (reflective) materials
	•	Dielectric materials (glass-like)
	•	Shadow rays and surface normals
	•	Gamma correction
	•	Anti-aliasing via stochastic sampling
	•	Defocus blur (depth of field)
	•	Progressive rendering with egui frontend (optional)

Key Features
	•	Book-accurate architecture: Reconstructs the core of Ray Tracing in One Weekend in safe, idiomatic Rust.
	•	Modular scene description: Uses a trait object abstraction (Hittable) to allow flexible scene composition.
	•	Material abstraction: Clean separation of physical material behavior via the Material trait.
	•	Multisampling camera: Camera logic supports customizable resolution, sample count, and field-of-view parameters.
	•	Progressive rendering interface: Double-buffered image updates using eframe and egui for interactive rendering preview.
	•	Double buffering: Safe and efficient lock-free pattern using atomic front buffer swapping for frame updates.

Repository Layout

├── camera.rs               # Ray generation, sampling, DoF, and rendering logic
├── hittable.rs             # Trait + hit record for intersection handling
├── hittable_list.rs        # Aggregates multiple hittables with closest hit selection
├── sphere.rs               # Basic sphere object implementing Hittable
├── material/
│   ├── lambertian.rs       # Diffuse reflection model
│   ├── metal.rs            # Reflective material
│   ├── dielectric.rs       # Glass-like refraction model
│   └── material.rs         # Trait and dyn material system
├── geometry/
│   ├── vec3.rs             # Core vector math
│   ├── ray.rs              # Ray representation
│   └── interval.rs         # Numerical range utility
├── double_buffer.rs        # Lock-free reader-writer pattern for progressive render
├── command.rs              # Render control commands/events
├── app.rs                  # egui front-end
├── main.rs                 # CLI and static image renderer

Rendering Modes

The program supports two main modes:

1. Static Render to PNG

Set ORIGINAL_RENDER_TO_PNG = true and run the app via CLI:

cargo run --release -- <resolution> <samples_per_pixel> <optional_filename_tag>

Images will be saved to the out/ directory.

2. Interactive egui Viewer

Set PROGRESSIVE_VIEWPORT = true to enable the GUI:

cargo run --release

Uses double-buffered frame updates and egui UI for pause/resume/interrupt control.

Goals & Next Steps
	•	Complete Book 1 (Ray Tracing in One Weekend)
	•	Begin Book 2 (Ray Tracing: The Next Week)
	•	Add BVH acceleration structure
	•	Add textures (checkered, image-based)
	•	Add lights and emissive materials
	•	Add support for volumetrics
	•	GPU acceleration (e.g., via wgpu, cuda, or metal)
	•	Explore adaptive sampling and denoising

Philosophy

This is not just about rendering pretty pictures — it’s about understanding the math and data structures that underpin modern graphics. I’m exploring each component deeply and experimenting with refactors, abstractions, and acceleration techniques along the way.