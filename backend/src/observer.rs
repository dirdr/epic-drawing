use image::{GrayImage, Rgb, RgbImage};
use imageproc::drawing::draw_filled_circle_mut;
use std::path::PathBuf;

use crate::linking::Component;

pub enum Step<'a> {
    Image(&'a GrayImage),
    Components(&'a GrayImage, &'a [Component]),
    Contour(&'a GrayImage, &'a [(i32, i32)]),
    DroppedComponents(&'a GrayImage, &'a [(u32, u32)], &'a [Vec<(u32, u32)>]),
    SampledPoints(&'a [(i32, i32)], &'a [usize], u32),
}

pub trait DebugObserver {
    fn on_step(&mut self, name: &str, step: Step<'_>);
}

pub struct NoOpObserver;

impl DebugObserver for NoOpObserver {
    #[inline(always)]
    fn on_step(&mut self, _name: &str, _step: Step<'_>) {}
}

#[cfg(not(target_arch = "wasm32"))]
pub struct FileObserver {
    output_dir: PathBuf,
    image_name: String,
    step_counter: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileObserver {
    pub fn new(output_dir: PathBuf, image_name: String) -> Self {
        Self {
            output_dir,
            image_name,
            step_counter: 0,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl DebugObserver for FileObserver {
    fn on_step(&mut self, name: &str, step: Step<'_>) {
        self.step_counter += 1;
        let path = self.output_dir.join(format!(
            "{}_{}_{}.png",
            self.image_name, self.step_counter, name
        ));

        let result = match step {
            Step::Image(img) => img.save(&path),
            Step::Components(edges, comps) => visualize_components(edges, comps).save(&path),
            Step::Contour(edges, contour) => visualize_contour(edges, contour).save(&path),
            Step::DroppedComponents(edges, largest, dropped) => {
                visualize_dropped_components(edges, largest, dropped).save(&path)
            }
            Step::SampledPoints(contour, sampled, size) => {
                visualize_sampled_points(contour, sampled, size).save(&path)
            }
        };

        match result {
            Ok(()) => println!("  [DEBUG] Saved: {}", path.display()),
            Err(e) => eprintln!("  [DEBUG] Failed to save {}: {}", path.display(), e),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn visualize_components(edges: &GrayImage, components: &[Component]) -> RgbImage {
    let (width, height) = edges.dimensions();
    let mut output = RgbImage::from_pixel(width, height, Rgb([0, 0, 0]));

    let colors = [
        Rgb([255, 0, 0]),
        Rgb([0, 255, 0]),
        Rgb([0, 0, 255]),
        Rgb([255, 255, 0]),
        Rgb([255, 0, 255]),
        Rgb([0, 255, 255]),
        Rgb([255, 128, 0]),
        Rgb([128, 0, 255]),
        Rgb([0, 255, 128]),
        Rgb([255, 128, 128]),
    ];

    for (i, comp) in components.iter().enumerate() {
        let color = colors[i % colors.len()];
        for &(x, y) in &comp.points {
            draw_filled_circle_mut(&mut output, (x, y), 1, color);
        }
    }

    output
}

#[cfg(not(target_arch = "wasm32"))]
fn visualize_contour(edges: &GrayImage, contour: &[(i32, i32)]) -> RgbImage {
    let (width, height) = edges.dimensions();
    let mut output = RgbImage::from_pixel(width, height, Rgb([0, 0, 0]));

    if contour.len() < 2 {
        return output;
    }

    let n = contour.len();
    for (i, &(x, y)) in contour.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;
        let r = (t * 255.0) as u8;
        let b = ((1.0 - t) * 255.0) as u8;
        let color = Rgb([r, 0, b]);
        draw_filled_circle_mut(&mut output, (x, y), 1, color);
    }

    output
}

#[cfg(not(target_arch = "wasm32"))]
fn visualize_dropped_components(
    edges: &GrayImage,
    largest: &[(u32, u32)],
    dropped: &[Vec<(u32, u32)>],
) -> RgbImage {
    let (width, height) = edges.dimensions();
    let mut output = RgbImage::from_pixel(width, height, Rgb([0, 0, 0]));

    for &(x, y) in largest {
        output.put_pixel(x, y, Rgb([0, 255, 0]));
    }

    for comp in dropped {
        for &(x, y) in comp {
            output.put_pixel(x, y, Rgb([255, 0, 0]));
        }
    }

    output
}

#[cfg(not(target_arch = "wasm32"))]
fn visualize_sampled_points(
    contour: &[(i32, i32)],
    sample_indices: &[usize],
    size: u32,
) -> RgbImage {
    let mut output = RgbImage::from_pixel(size, size, Rgb([0, 0, 0]));

    for &(x, y) in contour {
        if x >= 0 && x < size as i32 && y >= 0 && y < size as i32 {
            output.put_pixel(x as u32, y as u32, Rgb([50, 50, 50]));
        }
    }

    let n = sample_indices.len();
    for (i, &idx) in sample_indices.iter().enumerate() {
        if idx < contour.len() {
            let (x, y) = contour[idx];
            if x >= 0 && x < size as i32 && y >= 0 && y < size as i32 {
                let t = i as f32 / (n - 1).max(1) as f32;
                let r = (t * 255.0) as u8;
                let b = ((1.0 - t) * 255.0) as u8;
                draw_filled_circle_mut(&mut output, (x, y), 2, Rgb([r, 0, b]));
            }
        }
    }

    output
}
