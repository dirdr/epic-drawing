use image::GrayImage;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Copy, Default)]
pub struct LinkConfig {
    pub max_components: usize,
    pub max_link_distance_sq: i32,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub points: Vec<(i32, i32)>,
}

impl Component {
    pub fn start(&self) -> (i32, i32) {
        self.points[0]
    }

    pub fn end(&self) -> (i32, i32) {
        *self.points.last().unwrap()
    }

    fn dist_to_start(&self, point: (i32, i32)) -> i32 {
        let start = self.start();
        let dx = start.0 - point.0;
        let dy = start.1 - point.1;
        dx * dx + dy * dy
    }

    fn dist_to_end(&self, point: (i32, i32)) -> i32 {
        let end = self.end();
        let dx = end.0 - point.0;
        let dy = end.1 - point.1;
        dx * dx + dy * dy
    }

    fn reversed(&self) -> Self {
        Self {
            points: self.points.iter().copied().rev().collect(),
        }
    }
}

const NEIGHBORS_8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub fn find_all_components(edges: &GrayImage, min_points: usize) -> Vec<Component> {
    let (width, height) = edges.dimensions();
    let mut visited: HashSet<(u32, u32)> = HashSet::new();
    let mut components = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if edges.get_pixel(x, y).0[0] == 0 || visited.contains(&(x, y)) {
                continue;
            }

            let pixels = flood_fill(edges, x, y, &mut visited);
            if pixels.len() < min_points {
                continue;
            }

            let path = trace_path(edges, &pixels);
            components.push(Component { points: path });
        }
    }

    components
}

fn flood_fill(
    image: &GrayImage,
    start_x: u32,
    start_y: u32,
    visited: &mut HashSet<(u32, u32)>,
) -> Vec<(u32, u32)> {
    let (width, height) = image.dimensions();
    let mut pixels = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back((start_x, start_y));
    visited.insert((start_x, start_y));

    while let Some((x, y)) = queue.pop_front() {
        pixels.push((x, y));

        for (dx, dy) in NEIGHBORS_8 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let pos = (nx as u32, ny as u32);
                if image.get_pixel(pos.0, pos.1).0[0] > 0 && !visited.contains(&pos) {
                    visited.insert(pos);
                    queue.push_back(pos);
                }
            }
        }
    }

    pixels
}

fn count_neighbors(component: &HashSet<(u32, u32)>, x: u32, y: u32) -> usize {
    NEIGHBORS_8
        .iter()
        .filter(|(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            nx >= 0 && ny >= 0 && component.contains(&(nx as u32, ny as u32))
        })
        .count()
}

fn trace_path(image: &GrayImage, pixels: &[(u32, u32)]) -> Vec<(i32, i32)> {
    if pixels.is_empty() {
        return Vec::new();
    }

    let pixel_set: HashSet<(u32, u32)> = pixels.iter().copied().collect();

    let endpoints: Vec<(u32, u32)> = pixels
        .iter()
        .filter(|&&(x, y)| count_neighbors(&pixel_set, x, y) == 1)
        .copied()
        .collect();

    let start = endpoints.first().copied().unwrap_or(pixels[0]);

    let (width, height) = image.dimensions();
    let mut path = Vec::new();
    let mut visited: HashSet<(u32, u32)> = HashSet::new();
    let mut stack = vec![start];

    while let Some((x, y)) = stack.pop() {
        if visited.contains(&(x, y)) {
            continue;
        }

        visited.insert((x, y));
        path.push((x as i32, y as i32));

        for (dx, dy) in NEIGHBORS_8 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let pos = (nx as u32, ny as u32);
                if pixel_set.contains(&pos) && !visited.contains(&pos) {
                    stack.push(pos);
                }
            }
        }
    }

    path
}

pub fn link_components(components: Vec<Component>) -> Vec<(i32, i32)> {
    link_components_with_config(components, LinkConfig::default())
}

pub fn link_components_with_config(
    mut components: Vec<Component>,
    config: LinkConfig,
) -> Vec<(i32, i32)> {
    if components.is_empty() {
        return Vec::new();
    }

    if components.len() == 1 {
        return components.into_iter().next().unwrap().points;
    }

    components.sort_by_key(|c| std::cmp::Reverse(c.points.len()));

    if config.max_components > 0 && components.len() > config.max_components {
        components.truncate(config.max_components);
    }

    let mut result = Vec::new();
    let mut current = components.remove(0);

    result.extend_from_slice(&current.points);

    while !components.is_empty() {
        let current_end = current.end();

        let best = components
            .iter()
            .enumerate()
            .map(|(idx, comp)| {
                let dist_to_start = comp.dist_to_start(current_end);
                let dist_to_end = comp.dist_to_end(current_end);

                if dist_to_start <= dist_to_end {
                    (idx, dist_to_start, false)
                } else {
                    (idx, dist_to_end, true)
                }
            })
            .min_by_key(|(_, dist, _)| *dist);

        match best {
            Some((comp_idx, distance, should_reverse)) => {
                if config.max_link_distance_sq > 0 && distance > config.max_link_distance_sq {
                    break;
                }

                let mut next_comp = components.remove(comp_idx);

                if should_reverse {
                    next_comp = next_comp.reversed();
                }

                let connecting_points = bresenham_line(current_end, next_comp.start());
                result.extend(connecting_points);
                result.extend_from_slice(&next_comp.points);

                current = next_comp;
            }
            None => break,
        }
    }

    result
}

fn bresenham_line(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        points.push((x0, y0));

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }

    points
}

pub fn close_contour(mut contour: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    if contour.len() < 2 {
        return contour;
    }

    let first = contour[0];
    let last = *contour.last().unwrap();

    if first != last {
        let closing_line = bresenham_line(last, first);
        contour.extend(closing_line);
    }

    contour
}

fn count_neighbors_on_image(image: &GrayImage, x: u32, y: u32) -> usize {
    let (width, height) = image.dimensions();
    NEIGHBORS_8
        .iter()
        .filter(|(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            nx >= 0
                && nx < width as i32
                && ny >= 0
                && ny < height as i32
                && image.get_pixel(nx as u32, ny as u32).0[0] > 0
        })
        .count()
}

pub fn find_endpoints_on_image(image: &GrayImage) -> Vec<(u32, u32)> {
    let (width, height) = image.dimensions();
    let mut endpoints = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if image.get_pixel(x, y).0[0] > 0 && count_neighbors_on_image(image, x, y) == 1 {
                endpoints.push((x, y));
            }
        }
    }

    endpoints
}

fn draw_line_on_image(image: &mut GrayImage, from: (u32, u32), to: (u32, u32)) {
    let points = bresenham_line((from.0 as i32, from.1 as i32), (to.0 as i32, to.1 as i32));
    let (width, height) = image.dimensions();

    for (x, y) in points {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            image.put_pixel(x as u32, y as u32, image::Luma([255u8]));
        }
    }
}

pub fn bridge_gaps(image: &mut GrayImage, max_distance_sq: i32, max_bridges: usize) -> usize {
    let mut bridges_drawn = 0;

    loop {
        if bridges_drawn >= max_bridges {
            break;
        }

        let endpoints = find_endpoints_on_image(image);

        if endpoints.len() < 2 {
            break;
        }

        let mut best_pair: Option<(usize, usize, i32)> = None;

        for i in 0..endpoints.len() {
            for j in (i + 1)..endpoints.len() {
                let (x1, y1) = endpoints[i];
                let (x2, y2) = endpoints[j];

                let dx = x2 as i32 - x1 as i32;
                let dy = y2 as i32 - y1 as i32;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= 2 {
                    continue;
                }

                if dist_sq <= max_distance_sq {
                    match best_pair {
                        None => best_pair = Some((i, j, dist_sq)),
                        Some((_, _, best_dist)) if dist_sq < best_dist => {
                            best_pair = Some((i, j, dist_sq));
                        }
                        _ => {}
                    }
                }
            }
        }

        match best_pair {
            Some((i, j, _)) => {
                draw_line_on_image(image, endpoints[i], endpoints[j]);
                bridges_drawn += 1;
            }
            None => break,
        }
    }

    bridges_drawn
}

pub struct ComponentSelection {
    pub largest: Vec<(u32, u32)>,
    pub dropped: Vec<Vec<(u32, u32)>>,
}

pub fn find_all_components_raw(image: &GrayImage) -> ComponentSelection {
    let (width, height) = image.dimensions();
    let mut visited: HashSet<(u32, u32)> = HashSet::new();
    let mut all_components: Vec<Vec<(u32, u32)>> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if image.get_pixel(x, y).0[0] == 0 || visited.contains(&(x, y)) {
                continue;
            }
            let component = flood_fill(image, x, y, &mut visited);
            all_components.push(component);
        }
    }

    all_components.sort_by_key(|c| std::cmp::Reverse(c.len()));

    let largest = all_components.first().cloned().unwrap_or_default();
    let dropped = all_components.into_iter().skip(1).collect();

    ComponentSelection { largest, dropped }
}

fn find_largest_component(image: &GrayImage) -> Vec<(u32, u32)> {
    find_all_components_raw(image).largest
}

pub fn walk_contour(image: &GrayImage) -> Vec<(i32, i32)> {
    let largest = find_largest_component(image);

    if largest.is_empty() {
        return Vec::new();
    }

    let pixel_set: HashSet<(u32, u32)> = largest.iter().copied().collect();
    let (width, height) = image.dimensions();

    let start = largest
        .iter()
        .find(|&&(x, y)| count_neighbors_on_image(image, x, y) == 1)
        .copied()
        .unwrap_or(largest[0]);

    let mut path = Vec::new();
    let mut visited: HashSet<(u32, u32)> = HashSet::new();
    let mut current = start;

    loop {
        visited.insert(current);
        path.push((current.0 as i32, current.1 as i32));

        let unvisited_neighbors: Vec<(u32, u32)> = NEIGHBORS_8
            .iter()
            .filter_map(|(dx, dy)| {
                let nx = current.0 as i32 + dx;
                let ny = current.1 as i32 + dy;
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let pos = (nx as u32, ny as u32);
                    if pixel_set.contains(&pos) && !visited.contains(&pos) {
                        return Some(pos);
                    }
                }
                None
            })
            .collect();

        match unvisited_neighbors.len() {
            0 => break,
            1 => current = unvisited_neighbors[0],
            _ => {
                if path.len() >= 2 {
                    let prev = path[path.len() - 2];
                    let curr = (current.0 as i32, current.1 as i32);
                    let dir = (curr.0 - prev.0, curr.1 - prev.1);

                    current = *unvisited_neighbors
                        .iter()
                        .min_by_key(|&&(nx, ny)| {
                            let next_dir = (nx as i32 - curr.0, ny as i32 - curr.1);
                            -(dir.0 * next_dir.0 + dir.1 * next_dir.1)
                        })
                        .unwrap();
                } else {
                    current = unvisited_neighbors[0];
                }
            }
        }
    }

    path
}

pub fn bridge_and_walk(
    image: &mut GrayImage,
    max_bridge_distance: u32,
    max_bridges: usize,
) -> Vec<(i32, i32)> {
    let max_distance_sq = (max_bridge_distance * max_bridge_distance) as i32;
    bridge_gaps(image, max_distance_sq, max_bridges);
    walk_contour(image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bresenham_line() {
        let line = bresenham_line((0, 0), (3, 3));
        assert_eq!(line, vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
    }

    #[test]
    fn test_link_single_component() {
        let comp = Component {
            points: vec![(0, 0), (1, 1)],
        };
        let result = link_components(vec![comp]);
        assert_eq!(result, vec![(0, 0), (1, 1)]);
    }

    #[test]
    fn test_close_contour() {
        let contour = vec![(0, 0), (3, 0), (3, 3)];
        let closed = close_contour(contour);
        assert!(closed.len() > 3);
        assert_eq!(closed[0], (0, 0));
        assert_eq!(*closed.last().unwrap(), (0, 0));
    }

    #[test]
    fn test_link_with_max_components() {
        let comps = vec![
            Component {
                points: vec![(0, 0), (1, 1), (2, 2)],
            },
            Component {
                points: vec![(10, 10), (11, 11)],
            },
            Component {
                points: vec![(20, 20)],
            },
        ];
        let config = LinkConfig {
            max_components: 2,
            max_link_distance_sq: 0,
        };
        let result = link_components_with_config(comps, config);
        assert!(result.len() < 20);
    }

    #[test]
    fn test_link_with_max_distance() {
        let comps = vec![
            Component {
                points: vec![(0, 0), (1, 1)],
            },
            Component {
                points: vec![(1000, 1000)],
            },
        ];
        let config = LinkConfig {
            max_components: 0,
            max_link_distance_sq: 100, // Very small distance
        };
        let result = link_components_with_config(comps, config);
        assert_eq!(result.len(), 2);
    }
}
