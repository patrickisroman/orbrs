use image::{GenericImageView, ImageError, DynamicImage, ImageBuffer, Rgb};

// Types
pub type Point = (i32, i32);

#[derive(Debug)]
pub struct FastContext {
    offsets: Vec<Point>,
    fast: Vec<u8>,
    slow: Vec<u8>,
    radius: u32,
    n: u8
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

// Consts
const DEFAULT_THRESHOLD:i16 = 50;

// Methods
fn get_circle_slice(ctx: &FastContext, x: i32, y:i32) -> Vec<Point> {
    ctx.offsets
        .clone() // doesn't impact performance, but uhhh this feels wrong??
        .into_iter()
        .map(|(d_x, d_y)| (x + d_x, y + d_y))
        .collect()
}

pub fn fast(img: &image::GrayImage, fast_type: Option<FastType>, threshold: Option<i16>) -> Result<Vec<Point>, ImageError> {
    let threshold = threshold.unwrap_or(DEFAULT_THRESHOLD);
    let fast_type = fast_type.unwrap_or(FastType::TYPE_9_16);

    let ctx = fast_type.get_context();
    let indices_len = (ctx.fast.len() + ctx.slow.len()) as u8;
    let max_misses = indices_len - ctx.n;

    let mut fast_keypoint_matches = Vec::new();

    for y in ctx.radius .. img.height()-(ctx.radius+1) {
        'x_loop: for x in ctx.radius .. img.width()-(ctx.radius+1) {
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

            let mut similars = 0;
            for index in 0..ctx.fast.len() {
                let diff = (circle_pixels[ctx.fast[index] as usize] - center_pixel).abs();
                if diff < threshold {
                    similars += 1;
                    if similars > 1 {
                        continue 'x_loop;
                    }
                }
            }

            for index in 0..ctx.slow.len() {
                let diff = (circle_pixels[ctx.slow[index] as usize] - center_pixel).abs();

                if diff < threshold {
                    similars += 1;
                    if similars > max_misses {
                        continue 'x_loop;
                    }
                }
            }

            fast_keypoint_matches.push((x as i32, y as i32));
        }
    }

    Ok(fast_keypoint_matches)
}

pub fn draw_keypoints(img: &mut image::RgbImage, vec: &Vec<Point>) {
    let ctx = FastType::TYPE_9_16.get_context();
    let color = [255, 0, 0];
    for (x, y) in vec {
        for (c_x, c_y) in get_circle_slice(&ctx, *x, *y) {
            img.get_pixel_mut(c_x as u32, c_y as u32).0 = color;
        }
    }
}
