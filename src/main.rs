use image::{Rgb, RgbImage};
use std::path::PathBuf;

use structopt::StructOpt;

use crate::scene::Scene;
use crate::shapes::Ray;

mod intersection;
mod lights;
mod scene;
mod shapes;

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

    #[structopt(short, long)]
    enable_shadows: bool,
}

fn render(scene: &Scene, config: &Config) -> RgbImage {
    let viewplane_height = 2.0 * (scene.camera.height_angle / 2.0).tan(); // depth = 1
    let viewplane_width = viewplane_height * (config.width as f32 / config.height as f32);

    let mut output_image = RgbImage::new(config.width, config.height);
    for col in 0..output_image.width() {
        for row in 0..output_image.height() {
            let y = (config.height - 1 - row) as f32 / config.height as f32 - 0.5;
            let x = col as f32 / config.width as f32 - 0.5;

            let eye = glm::vec4(0.0, 0.0, 0.0, 1.0);
            let direction = glm::normalize(glm::vec4(
                viewplane_width * x,
                viewplane_height * y,
                -1.0,
                0.0,
            ));
            let camera_ray = Ray::new(eye, direction);
            let world_ray = camera_ray.transform(&scene.camera.inverse_view_matrix, false);

            let closest_intersection = scene
                .shapes
                .iter()
                .flat_map(|shape| shape.intersect(&world_ray))
                .min();

            let color = if let Some(intersection) = closest_intersection {
                lights::to_rgba(&lights::phong(scene, config, &intersection, &world_ray))
            } else {
                Rgb([0, 0, 0])
            };

            // NOTE: For rendering normals
            // let color = if let Some(intersection) = closest_intersection {
            //     let normal = glm::normalize(intersection.component_intersection.normal);
            //     Rgb([
            //         ((normal.x + 1.0) / 2.0 * 255.0) as u8,
            //         ((normal.y + 1.0) / 2.0 * 255.0) as u8,
            //         ((normal.z + 1.0) / 2.0 * 255.0) as u8,
            //     ])
            // } else {
            //     Rgb([0, 0, 0])
            // };

            output_image.put_pixel(col, row, color);
        }
    }

    output_image
}

fn main() {
    let config = Config::from_args();
    let tree_scene = scene::TreeScene::parse(&config.scene, &config.textures).unwrap();
    let scene = Scene::from(tree_scene);

    println!("{:#?}", scene);

    let output_image = render(&scene, &config);
    output_image.save(&config.output).unwrap();
}
