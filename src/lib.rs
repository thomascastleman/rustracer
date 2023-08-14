use anyhow::Result;
use image::RgbImage;
use raytracer::RayTracer;
use scene::{Scene, TreeScene};
use std::path::PathBuf;
use structopt::StructOpt;

mod intersection;
mod lights;
mod primitive;
pub mod raytracer;
pub mod scene;
mod shape;

/// Command-line options for the raytracer.
#[derive(Debug, StructOpt)]
#[structopt(name = "rustracer", about = "A Rust Raytracer")]
pub struct Config {
    /// Sets the width (pixels) of the output image
    #[structopt(short, long)]
    pub width: u32,
    /// Sets the height (pixels) of the output image
    #[structopt(short, long)]
    pub height: u32,
    /// Path to the .xml scenefile to render
    #[structopt(short, long, parse(from_os_str))]
    pub scene: PathBuf,
    /// Path where the output image should be rendered
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,
    /// Path of directory that texture images in the scenefile are relative to
    #[structopt(short, long, parse(from_os_str))]
    pub textures: PathBuf,
    /// Enable shadows
    #[structopt(long)]
    pub enable_shadows: bool,
    /// Enable reflective surfaces
    #[structopt(long)]
    pub enable_reflections: bool,
    /// Enable texture mapping
    #[structopt(long)]
    pub enable_texture: bool,
    /// Enable parallel processing of pixels
    #[structopt(long)]
    pub enable_parallelism: bool,
    /// Number of samples per pixel
    #[structopt(default_value = "1", long)]
    pub samples: u8,
}

/// Use the given configuration to produce a render of the indicated scenefile with the given parameters.
pub fn render_config<F: Fn() + Sync>(config: Config, pixel_finished: F) -> Result<RgbImage> {
    let tree_scene = TreeScene::parse(&config.scene, &config.textures)?;
    let scene = Scene::try_from(tree_scene)?;
    Ok(RayTracer::new(scene, config).render(pixel_finished))
}
