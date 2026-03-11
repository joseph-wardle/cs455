mod direct_lighting;
mod image;
mod intersection;
mod primary_ray_camera;
mod renderer;

pub use direct_lighting::shade_hit_with_directional_light;
pub use image::RenderImage;
pub use intersection::{HitObject, SurfaceHit, any_scene_hit, closest_scene_hit};
pub use primary_ray_camera::PrimaryRayCamera;
pub use renderer::{RenderConfig, Renderer};
