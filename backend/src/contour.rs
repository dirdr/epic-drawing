use image::{GrayImage, ImageBuffer, Luma, Rgb, RgbImage};
use imageproc::drawing::draw_filled_circle_mut;

use crate::linking::bridge_and_walk;

pub fn visualize_contour_ordered(
    edges: &ImageBuffer<Luma<u8>, Vec<u8>>,
    contour: &[(i32, i32)],
) -> RgbImage {
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

pub fn find_linked_contours(
    edges: &GrayImage,
    max_bridge_distance: u32,
    max_bridges: usize,
) -> Vec<(i32, i32)> {
    let mut skeleton = edges.clone();
    bridge_and_walk(&mut skeleton, max_bridge_distance, max_bridges)
}

pub fn find_linked_contours_with_image(
    edges: &GrayImage,
    max_bridge_distance: u32,
    max_bridges: usize,
) -> (GrayImage, Vec<(i32, i32)>) {
    let mut skeleton = edges.clone();
    let contour = bridge_and_walk(&mut skeleton, max_bridge_distance, max_bridges);
    (skeleton, contour)
}

pub fn visualize_linked_components(
    edges: &ImageBuffer<Luma<u8>, Vec<u8>>,
    linked_path: &[(i32, i32)],
    component_boundaries: &[usize],
) -> RgbImage {
    let (width, height) = edges.dimensions();
    let mut output = RgbImage::from_pixel(width, height, Rgb([0, 0, 0]));

    if linked_path.len() < 2 {
        return output;
    }

    let n = linked_path.len();

    for (i, &(x, y)) in linked_path.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;

        let is_connector = component_boundaries
            .windows(2)
            .any(|w| i >= w[0] && i < w[1]);

        let color = if is_connector {
            Rgb([0, 255, 0])
        } else {
            let r = (t * 255.0) as u8;
            let b = ((1.0 - t) * 255.0) as u8;
            Rgb([r, 0, b])
        };

        draw_filled_circle_mut(&mut output, (x, y), 1, color);
    }

    output
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_contour_visualization(
    edges: &ImageBuffer<Luma<u8>, Vec<u8>>,
    contour: &[(i32, i32)],
    path: &str,
) -> Result<(), image::ImageError> {
    let visualization = visualize_contour_ordered(edges, contour);
    visualization.save(path)
}
