use crate::scene::Scene;
use anyhow::Result;
use raytracer::RayTracer;
use std::path::PathBuf;
use structopt::StructOpt;

mod intersection;
mod lights;
mod primitive;
mod raytracer;
mod scene;
mod shape;

/// Command-line options for the raytracer.
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

    #[structopt(long)]
    enable_texture: bool,
}

/// Parses the CLI arguments, invokes the raytracer, and saves the output image, propagating errors.
fn run() -> Result<()> {
    let config = Config::from_args();
    let tree_scene = scene::TreeScene::parse(&config.scene, &config.textures)?;
    let scene = Scene::try_from(tree_scene)?;

    let output_image_path = config.output.clone();
    let raytracer = RayTracer::new(scene, config);
    raytracer.render().save(&output_image_path)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}
