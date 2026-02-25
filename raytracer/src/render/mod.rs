mod image;
mod intersection;
mod primary_ray_camera;
mod renderer;

pub use image::RenderImage;
pub use intersection::{SurfaceHit, closest_sphere_hit};
pub use primary_ray_camera::PrimaryRayCamera;
pub use renderer::{RenderConfig, Renderer};
