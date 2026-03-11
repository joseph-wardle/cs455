use crate::math::{ColorRgb, Direction3, Ray};
use crate::scene::Scene;

use super::{HitObject, SurfaceHit, any_scene_hit};

const SHADOW_RAY_BIAS: f64 = 1.0e-4;
const SHADOW_RAY_T_MIN: f64 = 1.0e-6;

pub fn shade_hit_with_directional_light(
    scene: &Scene,
    hit: SurfaceHit,
    normalized_direction_to_light: Direction3,
    incoming_ray_direction: Direction3,
) -> ColorRgb {
    let material = match hit.hit_object {
        HitObject::Sphere(sphere_index) => scene.spheres()[sphere_index].material(),
        HitObject::Triangle(triangle_index) => scene.triangles()[triangle_index].material(),
    };
    let lighting = scene.lighting();
    let ambient = lighting
        .ambient_light()
        .component_mul(material.diffuse_color())
        * material.ambient_weight();

    let unit_normal = hit.outward_normal.normalized();
    let ndotl = unit_normal.dot(normalized_direction_to_light).max(0.0);
    if ndotl <= 0.0 {
        return ambient;
    }

    let shadow_ray_origin = hit.point + (SHADOW_RAY_BIAS * unit_normal);
    let shadow_ray = Ray::new(shadow_ray_origin, normalized_direction_to_light);
    let hit_is_in_shadow = any_scene_hit(
        shadow_ray,
        scene.spheres(),
        scene.triangles(),
        SHADOW_RAY_T_MIN,
        f64::INFINITY,
        Some(hit.hit_object),
    );
    if hit_is_in_shadow {
        return ambient;
    }

    let diffuse = lighting
        .light_color()
        .component_mul(material.diffuse_color())
        * (material.diffuse_weight() * ndotl);

    let view_direction = (-incoming_ray_direction).normalized();
    let reflected_light = reflect_direction(-normalized_direction_to_light, unit_normal);
    let specular_alignment = reflected_light.dot(view_direction).max(0.0);
    let specular_strength =
        material.specular_weight() * specular_alignment.powf(material.gloss_exponent());
    let specular = lighting
        .light_color()
        .component_mul(material.specular_color())
        * specular_strength;

    ambient + diffuse + specular
}

fn reflect_direction(incident_direction: Direction3, unit_normal: Direction3) -> Direction3 {
    incident_direction - (2.0 * incident_direction.dot(unit_normal) * unit_normal)
}
