use image::imageops::FilterType;
use image::{GrayImage, ImageError};
use imageproc::edges::canny;
use imageproc::filter::gaussian_blur_f32;

use crate::config::ImageConfig;
use crate::linking::{bridge_gaps, close_contour, find_all_components_raw, walk_contour};
use crate::morphology::skeletonize;
use crate::observer::{DebugObserver, NoOpObserver, Step};

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub blur_sigma: f32,
    pub canny_low: f32,
    pub canny_high: f32,
    pub max_bridge_distance: u32,
    pub max_bridges: usize,
    pub num_points_sample: usize,
    pub target_size: f64,
}

impl PipelineConfig {
    pub fn from_image_config(config: &ImageConfig) -> Self {
        Self {
            blur_sigma: config.blur_sigma(),
            canny_low: config.canny_low_threshold(),
            canny_high: config.canny_high_threshold(),
            max_bridge_distance: config.max_link_distance() as u32,
            max_bridges: config.max_bridges(),
            num_points_sample: config.num_points_sample,
            target_size: config.target_size,
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            blur_sigma: 1.0,
            canny_low: 50.0,
            canny_high: 150.0,
            max_bridge_distance: 100,
            max_bridges: 50,
            num_points_sample: 500,
            target_size: 512.0,
        }
    }
}

pub struct Pipeline<O: DebugObserver = NoOpObserver> {
    observer: O,
}

impl Pipeline<NoOpObserver> {
    pub fn new() -> Self {
        Self {
            observer: NoOpObserver,
        }
    }
}

impl Default for Pipeline<NoOpObserver> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: DebugObserver> Pipeline<O> {
    pub fn with_observer(observer: O) -> Pipeline<O> {
        Pipeline { observer }
    }

    pub fn process(
        &mut self,
        image_bytes: &[u8],
        config: &PipelineConfig,
    ) -> Result<Vec<(i32, i32)>, ImageError> {
        let img = image::load_from_memory(image_bytes)?;
        let gray = img.to_luma8();
        self.observer.on_step("original", Step::Image(&gray));

        let resized = resize_stretch(&gray, 512, 512);
        self.observer.on_step("resized", Step::Image(&resized));

        let blurred = if config.blur_sigma > 0.0 {
            gaussian_blur_f32(&resized, config.blur_sigma)
        } else {
            resized
        };
        self.observer.on_step("blurred", Step::Image(&blurred));

        let edges = canny(&blurred, config.canny_low, config.canny_high);
        self.observer.on_step("edges", Step::Image(&edges));

        let skeleton = skeletonize(&edges);
        self.observer.on_step("skeleton", Step::Image(&skeleton));

        let mut bridged = skeleton.clone();
        let max_dist_sq = (config.max_bridge_distance * config.max_bridge_distance) as i32;
        let bridges = bridge_gaps(&mut bridged, max_dist_sq, config.max_bridges);
        self.observer.on_step("bridged", Step::Image(&bridged));

        let selection = find_all_components_raw(&bridged);
        self.observer.on_step(
            "components",
            Step::DroppedComponents(&bridged, &selection.largest, &selection.dropped),
        );

        let contour = walk_contour(&bridged);
        let closed = close_contour(contour);
        self.observer
            .on_step("contour", Step::Contour(&bridged, &closed));

        let sample_indices: Vec<usize> = (0..config.num_points_sample)
            .map(|i| {
                (i as f64 * closed.len() as f64 / config.num_points_sample as f64) as usize
                    % closed.len()
            })
            .collect();
        self.observer.on_step(
            "sampled",
            Step::SampledPoints(&closed, &sample_indices, 512),
        );

        if bridges > 0 {
            eprintln!("  Bridges drawn: {}", bridges);
        }

        Ok(closed)
    }
}

fn resize_stretch(img: &GrayImage, width: u32, height: u32) -> GrayImage {
    image::imageops::resize(img, width, height, FilterType::Lanczos3)
}
