use crate::{
    intersection::Intersection,
    raytracer::Ray,
    scene::{Scene, Texture},
    shape::Shape,
    Config,
};
use image::Rgb;

/// Offset from a point of intersecting that a recursive ray must be fired from
/// in order to avoid unwanted intersections with the intersected object itself.
pub const SELF_INTERSECT_OFFSET: f32 = 0.001;

pub fn phong(scene: &Scene, config: &Config, intersection: &Intersection, ray: &Ray) -> glm::Vec4 {
    let mut illumination = glm::vec4(0.0, 0.0, 0.0, 1.0);

    // First, add the ambient color of the material
    illumination =
        illumination + intersection.material.ambient * scene.global_lighting_coefficients.ka;

    let intersection_point = ray.at(intersection.component_intersection.t);
    let normal = intersection.component_intersection.normal;
    let intersection_to_camera = glm::normalize(-ray.direction);

    scene
        .lights
        .iter()
        .flat_map(|light| {
            if config.enable_shadows && !light.is_visible(&intersection_point, &scene.shapes) {
                return None;
            }

            let light_to_intersection = light.direction_to_point(&intersection_point);
            let intersection_to_light = -light_to_intersection;
            let mut diffuse_angle = glm::dot(normal, intersection_to_light);
            if diffuse_angle < 0.0 {
                diffuse_angle = 0.0;
            }

            let mut diffuse = glm::vec4(1.0, 1.0, 1.0, 1.0) * diffuse_angle;

            if config.enable_texture && intersection.material.texture.is_some() {
                let texture = intersection.material.texture.as_ref().unwrap();
                let texture_color =
                    uv_lookup(intersection.component_intersection.uv, texture, scene);

                diffuse = diffuse
                    * ((intersection.material.diffuse
                        * (1.0 - texture.blend)
                        * scene.global_lighting_coefficients.kd)
                        + (texture_color * texture.blend));
            } else {
                diffuse =
                    diffuse * scene.global_lighting_coefficients.kd * intersection.material.diffuse;
            }

            let mirror_direction = reflect_around(&light_to_intersection, &normal);
            let mut specular_angle = glm::dot(mirror_direction, intersection_to_camera);

            if specular_angle < 0.0 {
                specular_angle = 0.0;
            } else {
                specular_angle = specular_angle.powf(intersection.material.shininess);
            }

            let specular = intersection.material.specular
                * scene.global_lighting_coefficients.ks
                * specular_angle;

            Some(light.intensity_at(&intersection_point) * (diffuse + specular))
        })
        .fold(illumination, |acc, individual_light_illumination| {
            acc + individual_light_illumination
        })
}

fn clamp_intensity(intensity: f32) -> u8 {
    (255.0 * 1f32.min(0f32.max(intensity))) as u8
}

pub fn to_rgb(intensity: &glm::Vec4) -> Rgb<u8> {
    Rgb([
        clamp_intensity(intensity.x),
        clamp_intensity(intensity.y),
        clamp_intensity(intensity.z),
    ])
}

fn int_to_intensity(rgb_value: u8) -> f32 {
    rgb_value as f32 / 255.0
}

fn to_intensity(rgb: &Rgb<u8>) -> glm::Vec4 {
    glm::vec4(
        int_to_intensity(rgb[0]),
        int_to_intensity(rgb[1]),
        int_to_intensity(rgb[2]),
        1.0,
    )
}

// Calculates the attenuation of a light with the given attenuation function coefficients over the given distance
fn attenuation_over_distance(coefficients: &glm::Vec3, distance: f32) -> f32 {
    1f32.min(1.0 / (coefficients.z * distance.powi(2) + coefficients.y * distance + coefficients.x))
}

pub fn reflect_around(in_direction: &glm::Vec4, reflection_axis: &glm::Vec4) -> glm::Vec4 {
    glm::normalize(
        *in_direction - *reflection_axis * 2.0 * glm::dot(*in_direction, *reflection_axis),
    )
}

/// Converts a UV coordinate to the value of a texture at that coordinate.
fn uv_lookup(uv: (f32, f32), texture: &Texture, scene: &Scene) -> glm::Vec4 {
    let texture_image = scene
        .textures
        .get(&texture.filename)
        .expect("Tried to access unloaded texture");

    let (u, v) = uv;
    let column = (u * texture_image.width() as f32 * texture.repeat_u).floor() as u32
        % texture_image.width();
    let row = ((1.0 - v) * texture_image.height() as f32 * texture.repeat_v).floor() as u32
        % texture_image.height();

    to_intensity(texture_image.get_pixel(column, row))
}

#[derive(Debug)]
pub enum Light {
    Point {
        color: glm::Vector4<f32>,
        position: glm::Vector4<f32>,
        attenuation: glm::Vector3<f32>,
    },
    Directional {
        color: glm::Vector4<f32>,
        direction: glm::Vector4<f32>,
        attenuation: glm::Vector3<f32>,
    },
    Spot {
        color: glm::Vector4<f32>,
        position: glm::Vector4<f32>,
        direction: glm::Vector4<f32>,
        attenuation: glm::Vector3<f32>,
        penumbra: f32,
        angle: f32,
    },
}

impl Light {
    fn distance_to_point(&self, point: &glm::Vec4) -> Option<f32> {
        match self {
            Light::Directional { .. } => None,
            Light::Point { position, .. } | Light::Spot { position, .. } => {
                Some(glm::length(*position - *point))
            }
        }
    }

    fn is_visible(&self, point: &glm::Vec4, shapes: &[Shape]) -> bool {
        let to_point = self.direction_to_point(point);
        let point_to_light_ray = Ray::new(*point + (-to_point * SELF_INTERSECT_OFFSET), -to_point);
        let distance = self.distance_to_point(point);

        // The point is visible to the light if a ray from the point to the light
        // does not intersect with any other objects before hitting the light
        shapes
            .iter()
            .flat_map(|shape| shape.intersect(&point_to_light_ray))
            .filter(|intersection| match distance {
                // The light is infinitely far away, any intersection obstructs it
                None => true,
                // The light is some fixed distance away, look for intersections *closer* than it
                Some(distance) => intersection.component_intersection.t < distance,
            })
            .count()
            == 0
    }

    fn intensity_at(&self, point: &glm::Vec4) -> glm::Vec4 {
        let distance = self.distance_to_point(point);
        match self {
            Light::Directional { color, .. } => *color,
            Light::Point {
                color, attenuation, ..
            } => *color * attenuation_over_distance(attenuation, distance.unwrap()),
            Light::Spot {
                color,
                direction,
                attenuation,
                penumbra,
                angle,
                ..
            } => {
                let inner_angle = angle - penumbra;
                let attenuation = attenuation_over_distance(attenuation, distance.unwrap());

                let angle_between_spot_and_point = glm::acos(glm::dot(
                    glm::normalize(*direction),
                    self.direction_to_point(point),
                ));

                // If the angle to intersection is within the strongest part of the spot
                if angle_between_spot_and_point <= inner_angle {
                    return *color * attenuation;
                }

                // If the angle to intersection is fully outside the outermost angle, spot has no effect
                if angle_between_spot_and_point > *angle {
                    return glm::vec4(0.0, 0.0, 0.0, 1.0);
                }

                // Otherwise, the angle is between the inner angle and the outer, there is a falloff applied
                let falloff = {
                    let fraction_into_penumbra =
                        (angle_between_spot_and_point - inner_angle) / penumbra;
                    -2.0 * fraction_into_penumbra.powi(3) + 3.0 * fraction_into_penumbra.powi(2)
                };

                *color * attenuation * (1.0 - falloff)
            }
        }
    }

    /// Compute a vector from the light to the given point.
    fn direction_to_point(&self, point: &glm::Vec4) -> glm::Vec4 {
        glm::normalize(match self {
            Light::Directional { direction, .. } => *direction,
            Light::Point { position, .. } | Light::Spot { position, .. } => *point - *position,
        })
    }
}
