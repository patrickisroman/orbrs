use image::{GenericImageView, ImageError, DynamicImage, ImageBuffer, Rgb, GrayImage};
use imageproc::drawing::draw_line_segment_mut;
use cgmath::prelude::*;
use cgmath::{Rad};


// Consts
const DEFAULT_THRESHOLD:i16 = 50;

// Types
pub type Point = (i32, i32);

// Make a trait for FastKeypoint/OrientedFastKeypoint
#[derive(Debug, Clone, Copy)]
pub struct FastKeypoint {
    pub location: Point,
    pub score: u32,
    pub diff: i16,
    pub nms_dist: f64,
    pub moment: Moment
}

#[derive(Debug)]
pub struct FastContext {
    offsets: Vec<Point>,
    fast: Vec<u8>,
    slow: Vec<u8>,
    radius: u32,
    n: u32
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum FastType {
    TYPE_7_12,
    TYPE_9_16
}

impl FastType {
    pub fn get_context(&self) -> FastContext {
        match self {
            FastType::TYPE_7_12 => FastContext {
                offsets: vec![
                    (0, -2), (1, -2), (2,  -1), (2,   0),
                    (2,  1), (1,  2), (0,   2), (-1,  2),
                    (-2, 1), (-2, 0), (-2, -1), (-1, -2)
                ],
                fast: vec![0, 3, 6, 9],
                slow: vec![1, 2, 4, 5, 7, 8, 10, 11],
                radius: 2,
                n: 9
            },
            FastType::TYPE_9_16 => FastContext {
                offsets: vec![
                    (0, -3), (1,  -3), (2, - 2), (3,  -1),
                    (3,  0), (3,   1), (2,   2), (1,   3),
                    (0,  3), (-1,  3), (-2,  2), (-3,  1),
                    (-3, 0), (-3, -1), (-2, -2), (-1, -3) 
                ],
                fast: vec![0, 4, 8, 12],
                slow: vec![1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15],
                radius: 3,
                n: 12
            }
        }
    }
}

// Methods
fn get_circle_slice(ctx: &FastContext, x: i32, y:i32) -> Vec<Point> {
    ctx.offsets
        .clone() // doesn't impact performance, but uhhh this feels wrong??
        .into_iter()
        .map(|(d_x, d_y)| (x + d_x, y + d_y))
        .collect()
}

pub fn fast(img: &image::GrayImage, fast_type: Option<FastType>, threshold: Option<i16>) -> Result<Vec<FastKeypoint>, ImageError> {
    let threshold = threshold.unwrap_or(DEFAULT_THRESHOLD);
    let fast_type = fast_type.unwrap_or(FastType::TYPE_9_16);

    let ctx = fast_type.get_context();
    let indices_len = (ctx.fast.len() + ctx.slow.len()) as u32;
    let max_misses = indices_len - ctx.n;

    let mut fast_keypoint_matches = Vec::new();

    for y in ctx.radius .. img.height()-(ctx.radius+1) {
        'x_loop: for x in ctx.radius .. img.width()-(ctx.radius+1) {
            let mut score:i16 = 0;
            let center_pixel = img.get_pixel(x, y).0[0] as i16;
            let circle_pixels = get_circle_slice(&ctx, x as i32, y as i32)
                .into_iter()
                .map(|(p_x, p_y)| img.get_pixel(p_x as u32, p_y as u32).0[0] as i16)
                .collect::<Vec<i16>>();

            //
            // get_pixel performs an index check on each call
            // these are wasted cycles since we guarantee (x, y) in bounds
            // since (x, y) is derived, and the image buffer is immutable
            // TODO: use unsafe variant or direct buffer addressing
            //

            let mut similars:u32 = 0;
            for index in 0..ctx.fast.len() {
                let diff = (circle_pixels[ctx.fast[index] as usize] - center_pixel).abs();
                if diff < threshold {
                    similars += 1;
                    if similars > 1 {
                        continue 'x_loop;
                    }
                }
                score += diff;
            }

            for index in 0..ctx.slow.len() {
                let diff = (circle_pixels[ctx.slow[index] as usize] - center_pixel).abs();
                if diff < threshold {
                    similars += 1;
                    if similars > max_misses {
                        continue 'x_loop;
                    }
                }
                score += diff;
            }

            fast_keypoint_matches.push( FastKeypoint {
                location: (x as i32, y as i32),
                score: indices_len - similars,
                diff: score,
                nms_dist: 0.0,
                moment: Moment {
                    centroid: (0, 0),
                    moment: (0, 0),
                    rotation: 0.0
                }
            });
        }
    }

    // sort by diff comps
    fast_keypoint_matches.sort_by(|a, b| b.diff.cmp(&a.diff));
    Ok(fast_keypoint_matches)
}

//
// FAST Moment Calculations
//

#[derive(Debug, Clone, Copy)]
pub struct Moment {
    pub centroid: Point,
    pub moment: Point,
    pub rotation: f64
}

fn patch_moment(img: &GrayImage, x:u32, y:u32, x_moment:u32, y_moment:u32, moment_radius:Option<u32>) -> f32 {
    let moment_radius = moment_radius.unwrap_or(3);

    if x < moment_radius || y < moment_radius {
        return 1.0;
    }

    let mut patch_sum:u32 = 0;
    for mx in (x-moment_radius)..=(x+moment_radius) {
        for my in (y-moment_radius)..=(y+moment_radius) {
            patch_sum += mx.pow(x_moment) * my.pow(y_moment) * img.get_pixel(mx, my).0[0] as u32;
        }
    }

    patch_sum as f32
}

fn moment_centroid(img: &GrayImage, x:i32, y:i32, moment_radius:Option<u32>) -> Moment {
    // TODO weed out the repeated calculations here
    let p_m = patch_moment(img, x as u32, y as u32, 0, 0, moment_radius);
    let p_x = patch_moment(img, x as u32, y as u32, 1, 0, moment_radius);
    let p_y = patch_moment(img, x as u32, y as u32, 0, 1, moment_radius);

    let (mx, my) = (
        (p_x/p_m),
        (p_y/p_m)
    );

    let x_diff = (x as f32 - mx) as f64;
    let y_diff = (y as f32 - my) as f64;

    Moment {
        centroid: (x as i32, y as i32),
        moment: (mx.round() as i32, my.round() as i32),
        rotation: y_diff.atan2(x_diff)
    }
}

pub fn calculate_fast_centroids(img: &GrayImage, fast_keypoints: &mut Vec<FastKeypoint>) {
    for keypoint in fast_keypoints.iter_mut() {
        keypoint.moment = moment_centroid(img, keypoint.location.0, keypoint.location.1, None);
    }
}

pub fn draw_moments(img: &mut image::RgbImage, vec: &Vec<FastKeypoint>) {
    let ctx = FastType::TYPE_9_16.get_context();

    for k in vec {
        let score = (k.score - 12) as u8;
        let color = [50 * score, 0, 122];

        let start_point = k.location;

        let rotation_radians = Rad(k.moment.rotation);
        let dist = (score * 5) as f64;

        let end_point = (
            start_point.0 as f32 + (dist * Rad::cos(rotation_radians)).round() as f32,
            start_point.1 as f32 + (dist * Rad::sin(rotation_radians)).round() as f32
        );

        draw_line_segment_mut(
            img,
            (start_point.0 as f32, start_point.1 as f32),
            end_point,
            Rgb([0, 0, 0])
        );

        for (c_x, c_y) in get_circle_slice(&ctx, k.location.0, k.location.1) {
            img.get_pixel_mut(c_x as u32, c_y as u32).0 = color;
        }
    }
}