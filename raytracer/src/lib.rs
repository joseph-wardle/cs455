pub mod io;
pub mod math;
pub mod render;
pub mod scene;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::io::write_png;
use crate::render::{RenderConfig, Renderer};
use crate::scene::{SceneParseError, parse_scene_file};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("usage: raytracer <scene_file> <output_png>")]
    InvalidArguments,

    #[error("failed to parse scene '{path}': {source}")]
    ParseScene {
        path: String,
        #[source]
        source: SceneParseError,
    },

    #[error("failed to write image '{path}': {source}")]
    WriteImage {
        path: String,
        #[source]
        source: io::ImageWriteError,
    },
}

pub type AppResult<T> = Result<T, AppError>;

pub fn run() -> AppResult<()> {
    let cli_arguments = CliArguments::parse_from_env(std::env::args_os())?;
    render_scene_file_to_png(
        &cli_arguments.scene_file_path,
        &cli_arguments.output_png_path,
        RenderConfig::default(),
    )?;

    Ok(())
}

pub fn render_scene_file_to_png(
    scene_file_path: &Path,
    output_png_path: &Path,
    render_config: RenderConfig,
) -> AppResult<()> {
    let scene = parse_scene_file(scene_file_path).map_err(|source| AppError::ParseScene {
        path: scene_file_path.display().to_string(),
        source,
    })?;
    let renderer = Renderer::new(render_config);
    let image = renderer.render(&scene);

    write_png(output_png_path, &image).map_err(|source| AppError::WriteImage {
        path: output_png_path.display().to_string(),
        source,
    })?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArguments {
    scene_file_path: PathBuf,
    output_png_path: PathBuf,
}

impl CliArguments {
    fn parse_from_env(args: impl IntoIterator<Item = OsString>) -> AppResult<Self> {
        let mut args = args.into_iter();
        let _program_name = args.next();

        let scene_file_path = args.next().ok_or(AppError::InvalidArguments)?;
        let output_png_path = args.next().ok_or(AppError::InvalidArguments)?;

        if args.next().is_some() {
            return Err(AppError::InvalidArguments);
        }

        Ok(Self {
            scene_file_path: PathBuf::from(scene_file_path),
            output_png_path: PathBuf::from(output_png_path),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use image::RgbImage;

    use crate::render::RenderConfig;

    use super::CliArguments;
    use super::render_scene_file_to_png;

    const MAX_MEAN_ABS_CHANNEL_DELTA: f64 = 4.0;
    const LARGE_CHANNEL_DELTA_THRESHOLD: u8 = 32;
    const MAX_LARGE_DELTA_CHANNEL_FRACTION: f64 = 0.04;

    #[test]
    fn cli_parser_accepts_two_arguments() {
        let parsed = CliArguments::parse_from_env([
            "raytracer".into(),
            "input.scene".into(),
            "output.png".into(),
        ])
        .expect("two arguments should parse");

        assert_eq!(parsed.scene_file_path.to_string_lossy(), "input.scene");
        assert_eq!(parsed.output_png_path.to_string_lossy(), "output.png");
    }

    #[test]
    fn cli_parser_rejects_missing_arguments() {
        let parse_result = CliArguments::parse_from_env(["raytracer".into()]);
        assert!(parse_result.is_err());
    }

    #[test]
    fn render_scene_file_to_png_writes_png_with_expected_dimensions() {
        let fixture_scene_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../raytracer.part2.scene1.txt");
        let output_path = unique_temp_output_path();

        let render_result =
            render_scene_file_to_png(&fixture_scene_path, &output_path, RenderConfig::default());
        assert!(render_result.is_ok());

        let written_image = image::open(&output_path)
            .expect("rendered output should be readable PNG")
            .to_rgb8();
        assert_eq!(written_image.width(), 500);
        assert_eq!(written_image.height(), 500);

        let _ = std::fs::remove_file(output_path);
    }

    #[test]
    fn rendered_part2_scene1_matches_reference_image_with_tolerance() {
        let scene_path = fixture_path("raytracer.part2.scene1.txt");
        let reference_image_path = fixture_path("raytracer.part2.scene1.png");
        let output_path = unique_temp_output_path();

        let render_result =
            render_scene_file_to_png(&scene_path, &output_path, RenderConfig::default());
        assert!(render_result.is_ok());

        assert_images_match_reference_with_tolerance(&output_path, &reference_image_path);
        let _ = std::fs::remove_file(output_path);
    }

    #[test]
    fn rendered_part2_scene2_matches_reference_image_with_tolerance() {
        let scene_path = fixture_path("raytracer.part2.scene2.txt");
        let reference_image_path = fixture_path("raytracer.part2.scene2.png");
        let output_path = unique_temp_output_path();

        let render_result =
            render_scene_file_to_png(&scene_path, &output_path, RenderConfig::default());
        assert!(render_result.is_ok());

        assert_images_match_reference_with_tolerance(&output_path, &reference_image_path);
        let _ = std::fs::remove_file(output_path);
    }

    fn fixture_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../{file_name}"))
    }

    fn assert_images_match_reference_with_tolerance(actual_path: &Path, expected_path: &Path) {
        let actual = image::open(actual_path)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to open rendered image '{}': {error}",
                    actual_path.display()
                )
            })
            .to_rgb8();
        let expected = image::open(expected_path)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to open reference image '{}': {error}",
                    expected_path.display()
                )
            })
            .to_rgb8();

        assert_eq!(
            actual.dimensions(),
            expected.dimensions(),
            "rendered image and reference image dimensions must match"
        );

        let comparison = compare_images(&actual, &expected);
        assert!(
            comparison.mean_abs_channel_delta <= MAX_MEAN_ABS_CHANNEL_DELTA,
            "mean absolute channel delta {} exceeded allowed maximum {}",
            comparison.mean_abs_channel_delta,
            MAX_MEAN_ABS_CHANNEL_DELTA
        );
        assert!(
            comparison.large_delta_channel_fraction <= MAX_LARGE_DELTA_CHANNEL_FRACTION,
            "fraction of channels with delta > {} ({}) exceeded allowed maximum {}",
            LARGE_CHANNEL_DELTA_THRESHOLD,
            comparison.large_delta_channel_fraction,
            MAX_LARGE_DELTA_CHANNEL_FRACTION
        );
    }

    fn compare_images(actual: &RgbImage, expected: &RgbImage) -> ImageDifferenceSummary {
        let mut total_channel_delta: u64 = 0;
        let mut large_delta_channel_count: u64 = 0;

        for (actual_pixel, expected_pixel) in actual.pixels().zip(expected.pixels()) {
            for channel_index in 0..3 {
                let channel_delta =
                    actual_pixel[channel_index].abs_diff(expected_pixel[channel_index]) as u64;
                total_channel_delta += channel_delta;
                if channel_delta > LARGE_CHANNEL_DELTA_THRESHOLD as u64 {
                    large_delta_channel_count += 1;
                }
            }
        }

        let total_channel_count = (actual.width() as u64) * (actual.height() as u64) * 3;
        ImageDifferenceSummary {
            mean_abs_channel_delta: total_channel_delta as f64 / total_channel_count as f64,
            large_delta_channel_fraction: large_delta_channel_count as f64
                / total_channel_count as f64,
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct ImageDifferenceSummary {
        mean_abs_channel_delta: f64,
        large_delta_channel_fraction: f64,
    }

    fn unique_temp_output_path() -> PathBuf {
        let unix_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "raytracer-test-output-{}-{}.png",
            std::process::id(),
            unix_nanos
        ))
    }
}
