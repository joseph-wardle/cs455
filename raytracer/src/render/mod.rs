mod direct_lighting;
mod image;
mod intersection;
mod primary_ray_camera;
mod renderer;

pub use direct_lighting::shade_hit_with_directional_light;
pub use image::RenderImage;
pub use intersection::{SurfaceHit, any_sphere_hit, closest_sphere_hit};
pub use primary_ray_camera::PrimaryRayCamera;
pub use renderer::{RenderConfig, Renderer};
