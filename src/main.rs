use image::{Rgb, RgbImage};
use num_traits::Zero;
use std::path::PathBuf;

use structopt::StructOpt;

use crate::scene::Scene;
use crate::shapes::Ray;

mod intersection;
mod lights;
mod scene;
mod shapes;

const MAX_REFLECTION_DEPTH: u8 = 4;

#[derive(Debug, StructOpt)]
#[structopt(name = "rustracer", about = "A Rust Raytracer")]
pub struct Config {
    #[structopt(short, long)]
    width: u32,

    #[structopt(short, long)]
    height: u32,

    #[structopt(short, long, parse(from_os_str))]
    scene: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    textures: PathBuf,

    #[structopt(long)]
    enable_shadows: bool,

    #[structopt(long)]
    enable_reflections: bool,
}

struct RayTracer {
    scene: Scene,
    config: Config,
}

impl RayTracer {
    fn new(scene: Scene, config: Config) -> Self {
        Self { scene, config }
    }

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

    fn render(&self) -> RgbImage {
        let progress_bar =
            indicatif::ProgressBar::new((self.config.width * self.config.height) as u64);
        progress_bar.set_style(
            indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7} / {len:7} pixels",
            )
            .unwrap(),
        );

        let viewplane_height = 2.0 * (self.scene.camera.height_angle / 2.0).tan(); // depth = 1
        let viewplane_width =
            viewplane_height * (self.config.width as f32 / self.config.height as f32);

        let mut output_image = RgbImage::new(self.config.width, self.config.height);
        for col in 0..output_image.width() {
            for row in 0..output_image.height() {
                let y = (self.config.height - 1 - row) as f32 / self.config.height as f32 - 0.5;
                let x = col as f32 / self.config.width as f32 - 0.5;

                let eye = glm::vec4(0.0, 0.0, 0.0, 1.0);
                let direction = glm::normalize(glm::vec4(
                    viewplane_width * x,
                    viewplane_height * y,
                    -1.0,
                    0.0,
                ));

                let camera_ray = Ray::new(eye, direction);
                let world_ray = camera_ray.transform(&self.scene.camera.inverse_view_matrix, false);
                output_image.put_pixel(col, row, lights::to_rgba(&self.trace_ray(&world_ray, 0)));

                progress_bar.inc(1);
            }
        }

        progress_bar.finish();

        output_image
    }
}

fn main() {
    let config = Config::from_args();
    let tree_scene = scene::TreeScene::parse(&config.scene, &config.textures).unwrap();

    println!("{:#?}", tree_scene);

    let scene = Scene::from(tree_scene);

    println!("{:#?}", scene);

    let raytracer = RayTracer::new(scene, config);
    raytracer.render().save(&raytracer.config.output).unwrap();
}
