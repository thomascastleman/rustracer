use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use rustracer::{render_config, Config};
use std::path::PathBuf;

const BENCHMARK_IMG_WIDTH: u32 = 512;
const BENCHMARK_IMG_HEIGHT: u32 = 384;

/// Ratio of (number of pixels with diff) to (number of pixels), which, if exceeded,
/// will cause a test to fail and save the diff image.
const DIFF_THRESHOLD: f32 = 0.01;

/// Macro for generating a test case that renders a given scenefile with the rustracer
/// and compares this output with the corresponding benchmark image, succeeding if any
/// difference between the rendered images is acceptably negligible.
#[macro_export]
macro_rules! test_against_benchmark {
    ($directory:ident, $file:ident) => {
        ::paste::paste! {
            #[test]
            #[allow(non_snake_case)]
            fn [<$directory _ $file>]() {
                let tests_directory = ::std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
                let textures = tests_directory.join("textures");

                let scene = tests_directory.join(concat!(
                    "scenefiles/",
                    stringify!($directory),
                    "/",
                    stringify!($file),
                    ".xml"
                ));

                // NOTE: Output only saved if diff exceeds threshold (failed test)
                let output = tests_directory.join(concat!(
                    "diff_output/",
                    stringify!($directory),
                    "/",
                    stringify!($file),
                    ".png"
                ));

                let benchmark_output = tests_directory.join(concat!(
                    "benchmark_output/",
                    stringify!($directory),
                    "/",
                    stringify!($file),
                    ".png"
                ));

                $crate::common::render_and_diff(scene, textures, output, benchmark_output).unwrap();
            }
        }
    };
}

/// Renders a given scenefile under a common configuration, and compares the output with the benchmark.
pub fn render_and_diff(
    scene: PathBuf,
    textures: PathBuf,
    output: PathBuf,
    benchmark_output: PathBuf,
) -> Result<()> {
    let diff_image_path = output.clone();

    let config = Config {
        width: BENCHMARK_IMG_WIDTH,
        height: BENCHMARK_IMG_HEIGHT,
        scene,
        output,
        textures,
        enable_shadows: true,
        enable_reflections: true,
        enable_texture: true,
        enable_parallelism: true,
        samples: 1,
    };

    let image = render_config(config, || {})?;
    let benchmark_image = image::open(&benchmark_output)
        .with_context(|| {
            format!(
                "Failed to open benchmark image: {}",
                benchmark_output.display()
            )
        })?
        .into_rgb8();

    assert_eq!(image.width(), benchmark_image.width());
    assert_eq!(image.height(), benchmark_image.height());

    let (diff_image, pixels_with_diff) = calculate_diff_image(&image, &benchmark_image);

    let ratio_of_pixels_with_diff =
        pixels_with_diff as f32 / (image.width() * image.height()) as f32;

    if ratio_of_pixels_with_diff >= DIFF_THRESHOLD {
        diff_image
            .save(&diff_image_path)
            .with_context(|| format!("Failed to save diff image: {}", diff_image_path.display()))?;

        Err(anyhow::anyhow!(
            "Pixels with diff ({} pixels, {}%) exceeded threshold ({}%)",
            pixels_with_diff,
            ratio_of_pixels_with_diff * 100.0,
            DIFF_THRESHOLD * 100.0
        ))
    } else {
        Ok(())
    }
}

/// Calculates the pixel-by-pixel difference between two images, constructing a new image
/// that visually represents the difference, and indicating how many pixels differed.
fn calculate_diff_image(image: &RgbImage, benchmark_image: &RgbImage) -> (RgbImage, usize) {
    let mut diff_image = RgbImage::new(image.width(), image.height());
    let mut pixels_with_diff = 0;

    for ((x, y, image_pixel), benchmark_pixel) in
        image.enumerate_pixels().zip(benchmark_image.pixels())
    {
        // Scale a value from the range [0,255] to [LOWER_BOUND,255]
        fn amplify(value: u8) -> u8 {
            const LOWER_BOUND: u8 = 50;
            const NEW_RANGE: u8 = 255 - LOWER_BOUND;
            ((value as f32 / 255.0) * NEW_RANGE as f32) as u8 + LOWER_BOUND
        }

        // Calculate the absolute value of the difference between two values
        fn absolute_difference(value_a: u8, value_b: u8) -> u8 {
            (value_a as i16 - value_b as i16).unsigned_abs() as u8
        }

        if pixel_diff(image_pixel, benchmark_pixel) > 0 {
            pixels_with_diff += 1;

            // Use the difference between the red, green, and blue values as the color of each
            // of these values in the diff image, scaling them up to higher values to make
            // the diff easier to see even for small differences.
            diff_image.put_pixel(
                x,
                y,
                Rgb([
                    amplify(absolute_difference(image_pixel[0], benchmark_pixel[0])),
                    amplify(absolute_difference(image_pixel[1], benchmark_pixel[1])),
                    amplify(absolute_difference(image_pixel[2], benchmark_pixel[2])),
                ]),
            );
        } else {
            diff_image.put_pixel(x, y, Rgb([0, 0, 0]));
        }
    }

    (diff_image, pixels_with_diff)
}

/// Computes the difference between two RGB triples as their Euclidean distance.
fn pixel_diff(pixel_a: &Rgb<u8>, pixel_b: &Rgb<u8>) -> usize {
    ((pixel_a[0] as isize - pixel_b[0] as isize).pow(2)
        + (pixel_a[1] as isize - pixel_b[1] as isize).pow(2)
        + (pixel_a[2] as isize - pixel_b[2] as isize).pow(2)) as usize
}
