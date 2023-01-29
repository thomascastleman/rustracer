use image::{Rgb, RgbImage};
use num_traits::Zero;
use std::path::PathBuf;

use structopt::StructOpt;

use crate::scene::Scene;
use crate::shapes::Ray;

mod intersection;
mod scene;
mod shapes;

#[derive(Debug, StructOpt)]
#[structopt(name = "rustracer", about = "A Rust Raytracer")]
struct Config {
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
}

fn render(scene: &Scene, config: &Config) -> RgbImage {
    let viewplane_height = 2.0 * (scene.camera.height_angle / 2.0).tan(); // depth = 1
    let viewplane_width = viewplane_height * (config.width as f32 / config.height as f32);

    let mut output_image = RgbImage::new(config.width, config.height);
    for col in 0..output_image.width() {
        for row in 0..output_image.height() {
            let y = (config.height - 1 - row) as f32 / config.height as f32 - 0.5;
            let x = col as f32 / config.width as f32 - 0.5;

            let eye = glm::Vec4::zero();
            let direction = glm::normalize(glm::vec4(
                viewplane_width * x as f32,
                viewplane_height * y as f32,
                -1.0,
                0.0,
            ));
            let ray = Ray::new(eye, direction);

            for shape in &scene.shapes {}

            output_image.put_pixel(
                col,
                row,
                Rgb([
                    255,
                    (col % 255).try_into().unwrap(),
                    (row % 255).try_into().unwrap(),
                ]),
            );
        }
    }

    output_image
}

fn main() {
    let config = Config::from_args();
    let tree_scene = scene::TreeScene::parse(&config.scene, &config.textures).unwrap();
    let scene = Scene::from(tree_scene);

    let output_image = render(&scene, &config);
    output_image.save(&config.output).unwrap();

    println!("{:#?}", scene);
}
