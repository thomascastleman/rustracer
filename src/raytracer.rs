//! Core raytracing functionality.

use crate::lights;
use crate::scene::Scene;
use crate::Config;
use image::RgbImage;
use num_traits::Zero;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::mpsc::channel;

/// Total number of rays that will be traced (including camera ray) when
/// computing illumination for reflective materials.
const MAX_REFLECTION_DEPTH: u8 = 4;

/// A ray is like a beam that originates from a point and travels through the scene,
/// in a direction, possibly intersecting with an object(s) along its path.
#[derive(Debug)]
pub struct Ray {
    pub position: glm::Vec4,
    pub direction: glm::Vec4,
}

impl Ray {
    /// Constructs a new Ray from the given components.
    pub fn new(position: glm::Vec4, direction: glm::Vec4) -> Self {
        Self {
            position,
            direction,
        }
    }

    /// Transform the ray by the given transformation matrix. If `normalize_direction`
    /// is set, the new ray's `direction` will be guaranteed to be a unit vector.
    pub fn transform(&self, transformation: &glm::Mat4, normalize_direction: bool) -> Ray {
        let position = transformation.mul_v(&self.position);
        let mut direction = transformation.mul_v(&self.direction);

        if normalize_direction {
            direction = glm::normalize(direction);
        }

        Ray {
            position,
            direction,
        }
    }

    /// Convert a ray to object space by applying the given matrix and not normalizing the ray direction.
    pub fn to_object_space(&self, transformation: &glm::Mat4) -> Ray {
        self.transform(transformation, false)
    }

    /// Evaluate the ray at a given t-value, which indicates a point on the ray
    /// by acting as a scalar on the ray's direction vector.
    pub fn at(&self, t: f32) -> glm::Vec4 {
        self.position + self.direction * t
    }
}

/// A raytracer renders a given scene under a configuration.
pub struct RayTracer {
    scene: Scene,
    config: Config,
}

impl RayTracer {
    /// Constructs a new `RayTracer`.
    pub fn new(scene: Scene, config: Config) -> Self {
        Self { scene, config }
    }

    /// Trace the given ray into the raytracer's scene by determining if it intersects
    /// any objects, and if so, calculating what intensity contribution this ray makes.
    /// This may involve tracing further rays out from the point of intersection.
    fn trace_ray(&self, ray: &Ray, depth: u8) -> glm::Vec4 {
        // Look for the shape intersection with the minimum t-value (indicates closeness to the ray origin)
        let closest_intersection = &self
            .scene
            .shapes
            .iter()
            .flat_map(|shape| shape.intersect(ray))
            .min();

        match closest_intersection {
            Some(intersection) => {
                let color = lights::phong(&self.scene, &self.config, intersection, ray);

                if !self.config.enable_reflections
                    || glm::Vec4::zero() == intersection.material.reflective
                    || depth == MAX_REFLECTION_DEPTH
                {
                    // If there are no reflections enabled, the material isn't at all reflective,
                    // or we are at the maximum depth for recursively tracing rays, stop recurring.
                    color
                } else {
                    let reflected_direction = lights::reflect_around(
                        &ray.direction,
                        &intersection.component_intersection.normal,
                    );
                    let reflected_ray = Ray::new(
                        ray.at(intersection.component_intersection.t)
                            + (reflected_direction * lights::SELF_INTERSECT_OFFSET),
                        reflected_direction,
                    );
                    let reflected_light = intersection.material.reflective
                        * self.scene.global_lighting_coefficients.ks
                        * self.trace_ray(&reflected_ray, depth + 1);

                    // Use the color from the original ray, but add the contribution of a
                    // ray that has been reflected off the intersected surface
                    color + reflected_light
                }
            }
            // There is no intersection, so there is no illumination from this ray
            None => glm::vec4(0.0, 0.0, 0.0, 1.0),
        }
    }

    /// Produces an image by rendering the raytracer's scene.
    pub fn render(&self) -> RgbImage {
        let progress_bar =
            indicatif::ProgressBar::new((self.config.width * self.config.height) as u64);
        progress_bar.set_style(
            indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {percent}% {pos:>7} / {len:7} pixels",
            )
            .unwrap(),
        );

        let viewplane_height = 2.0 * (self.scene.camera.height_angle / 2.0).tan(); // depth = 1
        let viewplane_width =
            viewplane_height * (self.config.width as f32 / self.config.height as f32);

        let mut output_image = RgbImage::new(self.config.width, self.config.height);
        let output_width = output_image.width();

        // Renders a single pixel at the given 1-dimensional index in the image,
        // returning its row/column position as well as the computed pixel color.
        let render_pixel = |pixel_index| {
            // Convert pixel index to 2D discrete image coordinates
            let row = pixel_index / output_width;
            let col = pixel_index % output_width;

            // Convert the image coordinates to continuous view plane coordinates
            let y = (self.config.height - 1 - row) as f32 / self.config.height as f32 - 0.5;
            let x = col as f32 / self.config.width as f32 - 0.5;

            // Determine the direction from the camera to the pixel
            let eye = glm::vec4(0.0, 0.0, 0.0, 1.0);
            let direction = glm::normalize(glm::vec4(
                viewplane_width * x,
                viewplane_height * y,
                -1.0,
                0.0,
            ));

            // Construct a ray from the camera through this pixel, and trace it into the scene
            let camera_ray = Ray::new(eye, direction);
            let world_ray = camera_ray.transform(&self.scene.camera.inverse_view_matrix, false);
            let pixel_color = lights::to_rgb(&self.trace_ray(&world_ray, 0));

            // Increment the progress bar
            progress_bar.inc(1);

            (col, row, pixel_color)
        };

        let all_pixel_indices = 0..(output_image.width() * output_image.height());

        if self.config.enable_parallelism {
            let (sender, receiver) = channel();

            // Render all pixels in parallel, sending output to the image writer
            all_pixel_indices
                .into_par_iter()
                .for_each_with(sender, |pixel_writer, pixel_index| {
                    pixel_writer.send(render_pixel(pixel_index)).unwrap();
                });

            // Receive the pixel data and write it to the image buffer
            for (x, y, color) in receiver.iter() {
                output_image.put_pixel(x, y, color);
            }
        } else {
            for (x, y, color) in all_pixel_indices.map(render_pixel) {
                output_image.put_pixel(x, y, color);
            }
        };

        progress_bar.finish();

        output_image
    }
}
