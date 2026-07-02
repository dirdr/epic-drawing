use crate::config::{ImageConfig, parse_config};
use crate::fourier::{EquationData, compute_coefficients};
use crate::pipeline::{Pipeline, PipelineConfig};
use anyhow::{Context, Result};
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize)]
pub struct ImageData {
    pub name: String,
    pub equation_data: EquationData,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImagesResponse {
    pub images: Vec<ImageData>,
}

static IMAGES_CACHE: OnceLock<ImagesResponse> = OnceLock::new();

pub fn get_cached_images() -> Result<&'static ImagesResponse> {
    IMAGES_CACHE.get_or_init(|| {
        initialize_cache().unwrap_or_else(|e| {
            eprintln!("Failed to initialize image cache: {}", e);
            ImagesResponse { images: vec![] }
        })
    });

    let response = IMAGES_CACHE.get().unwrap();
    if response.images.is_empty() {
        anyhow::bail!("No images were successfully loaded");
    }
    Ok(response)
}

fn initialize_cache() -> Result<ImagesResponse> {
    let config_str = include_str!("../images.toml");
    let config = parse_config(config_str).context("Failed to parse config")?;

    let mut images = Vec::new();
    for img_config in &config.image {
        match process_image_config(img_config) {
            Ok(image_data) => images.push(image_data),
            Err(e) => {
                eprintln!("Warning: Skipping image '{}': {}", img_config.name, e);
            }
        }
    }

    if images.is_empty() {
        anyhow::bail!("No images were successfully processed");
    }

    Ok(ImagesResponse { images })
}

#[derive(Debug, Clone)]
pub struct ContourTransform {
    sampled: Vec<(f64, f64)>,
}

impl ContourTransform {
    pub fn from_contour(contour: &[(i32, i32)], num_points: usize) -> Self {
        let sampled = sample_points(contour, num_points);
        Self { sampled }
    }

    pub fn center(mut self) -> Self {
        center_points(&mut self.sampled);
        self
    }

    pub fn scale_to_size(mut self, target_size: f64) -> Self {
        scale_points(&mut self.sampled, target_size);
        self
    }

    pub fn into_xy_vectors(self) -> (Vec<f64>, Vec<f64>) {
        self.sampled.into_iter().unzip()
    }
}

fn sample_points(contour: &[(i32, i32)], num_points: usize) -> Vec<(f64, f64)> {
    if contour.is_empty() {
        return vec![(0.0, 0.0)];
    }

    let mut sampled = Vec::with_capacity(num_points);
    let step = contour.len() as f64 / num_points as f64;

    for i in 0..num_points {
        let idx = (i as f64 * step) as usize % contour.len();
        let (x, y) = contour[idx];
        sampled.push((x as f64, y as f64));
    }

    sampled
}

fn center_points(points: &mut [(f64, f64)]) {
    if points.is_empty() {
        return;
    }

    let center_x = points.iter().map(|(x, _)| x).sum::<f64>() / points.len() as f64;
    let center_y = points.iter().map(|(_, y)| y).sum::<f64>() / points.len() as f64;

    for (x, y) in points.iter_mut() {
        *x -= center_x;
        *y = center_y - *y;
    }
}

fn scale_points(points: &mut [(f64, f64)], target_size: f64) {
    if points.is_empty() {
        return;
    }

    let max_extent = points
        .iter()
        .map(|(x, y)| x.abs().max(y.abs()))
        .fold(0.0_f64, f64::max);

    if max_extent > 0.0 {
        let scale = target_size / max_extent;
        for (x, y) in points.iter_mut() {
            *x *= scale;
            *y *= scale;
        }
    }
}

fn process_image_config(config: &ImageConfig) -> Result<ImageData> {
    let image_bytes: &[u8] = match config.name.as_str() {
        "elephant" => include_bytes!("../resources/input/elephant.jpg"),
        _ => anyhow::bail!("Unknown image: {}", config.name),
    };

    let pipeline_config = PipelineConfig::from_image_config(config);
    let mut pipeline = Pipeline::new();

    let contour = pipeline
        .process(image_bytes, &pipeline_config)
        .context("Failed to process image")?;

    if contour.is_empty() {
        anyhow::bail!("No contour detected in image");
    }

    let (x_points, y_points) =
        sample_contour_points(&contour, config.num_points_sample, config.target_size);

    let max_coeffs = if config.max_coefficients == 0 {
        None
    } else {
        Some(config.max_coefficients)
    };

    let coefficients = compute_coefficients(x_points, y_points, max_coeffs);

    Ok(ImageData {
        name: config.name.clone(),
        equation_data: EquationData {
            coefficients,
            period: std::f64::consts::TAU,
        },
    })
}

pub fn sample_contour_points(
    contour: &[(i32, i32)],
    num_points: usize,
    target_size: f64,
) -> (Vec<f64>, Vec<f64>) {
    ContourTransform::from_contour(contour, num_points)
        .center()
        .scale_to_size(target_size)
        .into_xy_vectors()
}
