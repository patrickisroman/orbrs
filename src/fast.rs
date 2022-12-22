use cgmath::{prelude::*, Rad};
use image::{GrayImage, ImageError, Rgba};
use imageproc::drawing::draw_line_segment_mut;

use crate::common;
use common::*;

// Consts
const DEFAULT_FAST_THRESHOLD: i32 = 50;

#[derive(Debug, Clone, Copy)]
pub struct FastKeypoint {
    pub location: Point,
    pub score: i32,
    pub nms_dist: usize,
    pub moment: Moment,
}

impl Matchable for FastKeypoint {
    fn distance(&self, other: &FastKeypoint) -> usize {
        let ((ax, ay), (bx, by)) = (self.location, other.location);
        ((ax - bx).pow(2) as f32 + (ay - by).pow(2) as f32).sqrt() as usize
    }
}

#[derive(Debug)]
pub struct FastContext {
    offsets: Vec<Point>,
    idx: Vec<usize>,
    cmp: Vec<i32>,
    radius: u32,
    n: u32,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum FastType {
    TYPE_7_12,
    TYPE_9_16,
}

impl FastType {
    pub fn get_context(&self) -> FastContext {
        match self {
            FastType::TYPE_7_12 => FastContext {
                offsets: vec![
                    (0, -2),
                    (1, -2),
                    (2, -1),
                    (2, 0),
                    (2, 1),
                    (1, 2),
                    (0, 2),
                    (-1, 2),
                    (-2, 1),
                    (-2, 0),
                    (-2, -1),
                    (-1, -2),
                ],
                idx: vec![0, 6, 3, 9, 1, 2, 4, 5, 7, 8, 10, 11],
                cmp: vec![1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3],
                radius: 3,
                n: 9,
            },
            FastType::TYPE_9_16 => FastContext {
                offsets: vec![
                    (0, -3),
                    (1, -3),
                    (2, -2),
                    (3, -1),
                    (3, 0),
                    (3, 1),
                    (2, 2),
                    (1, 3),
                    (0, 3),
                    (-1, 3),
                    (-2, 2),
                    (-3, 1),
                    (-3, 0),
                    (-3, -1),
                    (-2, -2),
                    (-1, -3),
                ],
                idx: vec![0, 8, 4, 12, 1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15],
                cmp: vec![1, 1, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
                radius: 4,
                n: 12,
            },
        }
    }
}

pub fn fast(
    img: &image::GrayImage,
    fast_type: Option<FastType>,
    threshold: Option<i32>,
) -> Result<Vec<FastKeypoint>, ImageError> {
    let threshold = threshold.unwrap_or(DEFAULT_FAST_THRESHOLD);
    let fast_type = fast_type.unwrap_or(FastType::TYPE_9_16);

    let ctx = fast_type.get_context();

    let mut fast_keypoint_matches = Vec::<FastKeypoint>::new();

    for y in ctx.radius..img.height() - ctx.radius {
        'x_loop: for x in ctx.radius..img.width() - ctx.radius {
            let center_pixel = img.get_pixel(x, y).0[0] as i32;
            let x = x as i32;
            let y = y as i32;
            let point: Point = (x, y);

            let mut score: i32 = 0;
            let mut similars: i32 = 0;

            for idx in 0..ctx.offsets.len() {
                let px_idx = ctx.idx[idx];
                let px = img
                    .get_pixel(
                        (x + ctx.offsets[px_idx].0) as u32,
                        (y + ctx.offsets[px_idx].1) as u32,
                    )
                    .0[0] as i32;
                let diff = (px - center_pixel).abs();

                if diff < threshold {
                    similars += 1;
                    if similars > ctx.cmp[idx] {
                        continue 'x_loop;
                    }
                } else {
                    score += diff;
                }
            }

            let moment = moment_centroid(img, &point, None);
            fast_keypoint_matches.push(FastKeypoint {
                location: point,
                score: score,
                nms_dist: 0,
                moment: moment,
            });
        }
    }

    // sort by score comps
    fast_keypoint_matches.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(fast_keypoint_matches)
}

//
// Oriented FAST Moment Calculations
//

#[derive(Debug, Clone, Copy)]
pub struct Moment {
    pub centroid: Point,
    pub moment: Point,
    pub rotation: f64,
}

fn patch_moment(
    img: &GrayImage,
    x: u32,
    y: u32,
    x_moment: u32,
    y_moment: u32,
    radius: Option<u32>,
) -> f64 {
    let radius = radius.unwrap_or(5);

    if x < radius || y < radius || x + radius >= img.width() || y + radius >= img.height() {
        return 1.0;
    }

    let mut patch_sum: u32 = 0;
    for mx in (x - radius)..=(x + radius) {
        for my in (y - radius)..=(y + radius) {
            let coefficient = match (x_moment, y_moment) {
                (0, 0) => 1,
                (0, 1) => my,
                (1, 0) => mx,
                _ => 0,
            };
            patch_sum += coefficient * img.get_pixel(mx, my).0[0] as u32;
        }
    }

    patch_sum as f64
}

fn moment_centroid(img: &GrayImage, point: &Point, moment_radius: Option<u32>) -> Moment {
    let p_m = patch_moment(img, point.0 as u32, point.1 as u32, 0, 0, moment_radius);
    let p_x = patch_moment(img, point.0 as u32, point.1 as u32, 1, 0, moment_radius);
    let p_y = patch_moment(img, point.0 as u32, point.1 as u32, 0, 1, moment_radius);

    let (mx, my) = ((p_x / p_m), (p_y / p_m));

    let x_diff = (point.0 as f64 - mx) as f64;
    let y_diff = (point.1 as f64 - my) as f64;

    Moment {
        centroid: point.clone(),
        moment: (mx.round() as i32, my.round() as i32),
        rotation: y_diff.atan2(x_diff),
    }
}

pub fn draw_moments(img: &mut image::RgbaImage, vec: &Vec<FastKeypoint>) {
    let ctx = FastType::TYPE_9_16.get_context();

    for k in vec {
        let score = (k.score / 300) as u8;
        let color = [score, 0, 122, 125];

        let start_point = k.location;

        let rotation_radians = Rad(k.moment.rotation);
        let dist = (score * 5) as f64;

        let end_point = (
            start_point.0 as f32 + (dist * Rad::cos(rotation_radians)).round() as f32,
            start_point.1 as f32 + (dist * Rad::sin(rotation_radians)).round() as f32,
        );

        draw_line_segment_mut(
            img,
            (start_point.0 as f32, start_point.1 as f32),
            end_point,
            Rgba([0, 0, 0, 125]),
        );

        // draw circle
        ctx.offsets.clone().into_iter().for_each(|(dx, dy)| {
            img.get_pixel_mut((k.location.0 + dx) as u32, (k.location.1 + dy) as u32)
                .0 = color;
        });
    }
}
