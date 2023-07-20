//! A raytracer, which consumes XML scenefiles ([as used in Brown's CS1230](https://github.com/BrownCSCI1230/scenefiles))
//! and produces images that portray a 3D view of the scenes.

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
    /// Sets the width (pixels) of the output image
    #[structopt(short, long)]
    width: u32,
    /// Sets the height (pixels) of the output image
    #[structopt(short, long)]
    height: u32,
    /// Path to the .xml scenefile to render
    #[structopt(short, long, parse(from_os_str))]
    scene: PathBuf,
    /// Path where the output image should be rendered
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
    /// Path of directory that texture images in the scenefile are relative to
    #[structopt(short, long, parse(from_os_str))]
    textures: PathBuf,
    /// Enable shadows
    #[structopt(long)]
    enable_shadows: bool,
    /// Enable reflective surfaces
    #[structopt(long)]
    enable_reflections: bool,
    /// Enable texture mapping
    #[structopt(long)]
    enable_texture: bool,
    /// Enable parallel processing of pixels
    #[structopt(long)]
    enable_parallelism: bool,
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
