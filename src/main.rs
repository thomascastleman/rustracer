//! A raytracer, which consumes XML scenefiles ([as used in Brown's CS1230](https://github.com/BrownCSCI1230/scenefiles))
//! and produces images that portray a 3D view of the scenes.

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rustracer::Config;
use structopt::StructOpt;

/// Parses the CLI arguments, invokes the raytracer, and saves the output image, propagating errors.
fn run() -> Result<()> {
    let config = Config::from_args();

    println!(
        "Rendering {} as {}x{} image",
        config.scene.display(),
        config.width,
        config.height
    );

    let progress_bar = ProgressBar::new((config.width * config.height) as u64);

    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {percent}% {pos:>7} / {len:7} pixels",
        )
        .unwrap(),
    );

    let output_image_path = config.output.clone();
    let output_image = rustracer::render_config(config, || {
        progress_bar.inc(1);
    })?;

    progress_bar.finish();

    output_image.save(&output_image_path)?;

    println!("Output saved as {}", output_image_path.display());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}
