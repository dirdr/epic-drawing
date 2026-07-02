use image::{GrayImage, Luma};
use imageproc::distance_transform::Norm;
use imageproc::morphology::close;

pub fn morphological_close(image: &GrayImage, iterations: u8) -> GrayImage {
    if iterations == 0 {
        return image.clone();
    }

    let mut result = image.clone();
    for _ in 0..iterations {
        result = close(&result, Norm::LInf, 1);
    }
    result
}

pub fn skeletonize(image: &GrayImage) -> GrayImage {
    let mut current = image.clone();
    let mut changed = true;

    while changed {
        changed = false;

        let to_remove = find_removable_pixels(&current, true);
        if !to_remove.is_empty() {
            changed = true;
            for (x, y) in &to_remove {
                current.put_pixel(*x, *y, Luma([0]));
            }
        }

        let to_remove = find_removable_pixels(&current, false);
        if !to_remove.is_empty() {
            changed = true;
            for (x, y) in &to_remove {
                current.put_pixel(*x, *y, Luma([0]));
            }
        }
    }

    current
}

fn find_removable_pixels(image: &GrayImage, first_subiteration: bool) -> Vec<(u32, u32)> {
    let (width, height) = image.dimensions();
    let mut removable = Vec::new();

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if image.get_pixel(x, y).0[0] == 0 {
                continue;
            }

            let p2 = is_foreground(image, x, y - 1);
            let p3 = is_foreground(image, x + 1, y - 1);
            let p4 = is_foreground(image, x + 1, y);
            let p5 = is_foreground(image, x + 1, y + 1);
            let p6 = is_foreground(image, x, y + 1);
            let p7 = is_foreground(image, x - 1, y + 1);
            let p8 = is_foreground(image, x - 1, y);
            let p9 = is_foreground(image, x - 1, y - 1);

            let neighbors = [p2, p3, p4, p5, p6, p7, p8, p9];

            let b = neighbors.iter().filter(|&&n| n).count();
            if !(2..=6).contains(&b) {
                continue;
            }

            let a = count_transitions(&neighbors);
            if a != 1 {
                continue;
            }

            if first_subiteration {
                if (p8 || p2) && p6 && p4 {
                    continue;
                }
            } else if (p6 || p4) && p8 && p2 {
                continue;
            }

            removable.push((x, y));
        }
    }

    removable
}

#[inline]
fn is_foreground(image: &GrayImage, x: u32, y: u32) -> bool {
    image.get_pixel(x, y).0[0] > 0
}

fn count_transitions(neighbors: &[bool; 8]) -> usize {
    (0..8)
        .filter(|&i| !neighbors[i] && neighbors[(i + 1) % 8])
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeletonize_simple_line() {
        let mut img = GrayImage::new(10, 10);
        for x in 2..8 {
            for y in 4..7 {
                img.put_pixel(x, y, Luma([255]));
            }
        }

        let skeleton = skeletonize(&img);

        let height_count = (0..10)
            .filter(|&y| skeleton.get_pixel(5, y).0[0] > 0)
            .count();
        assert!(height_count <= 2);
    }

    #[test]
    fn test_morphological_close_bridges_gap() {
        let mut img = GrayImage::new(10, 10);
        img.put_pixel(3, 5, Luma([255]));
        img.put_pixel(5, 5, Luma([255]));

        let closed = morphological_close(&img, 1);
        assert!(closed.get_pixel(4, 5).0[0] > 0);
    }

    #[test]
    fn test_close_zero_iterations() {
        let img = GrayImage::new(10, 10);
        let result = morphological_close(&img, 0);
        assert_eq!(img.dimensions(), result.dimensions());
    }
}
