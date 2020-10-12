extern crate image;
use image::{GenericImageView, ImageError};

// Types
pub type Point = (i32, i32);

pub struct FastContext {
    offsets: Vec<Point>,
    slow: Vec<u8>,
    fast: Vec<u8>,
    radius: u32,
    n: u8
}

// TODO: implement more types
pub enum FastType {
    TYPE_9_16
}

impl FastType {
    pub fn get_context(&self) -> FastContext {
        match self {
            TYPE_9_16 => FastContext {
                offsets: vec![
                    (0, -3), (1, -3),  (2, -2),  (3, -1),
                    (3, 0),  (3, 1),   (2, 2),   (1, 3),
                    (0, 3),  (-1, 3),  (-2, 2),  (-3, 1),
                    (-3, 0), (-3, -1), (-2, -2), (-1, -3) 
                ],
                slow: vec![0, 4, 8, 12],
                fast: vec![1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15],
                radius: 3,
                n: 12
            }
        }
    }
}

// Consts
const DEFAULT_THRESHOLD:i16 = 40;

// Methods
fn get_circle_slice(ctx: &FastContext, x: i32, y:i32) -> Vec<Point> {
    ctx.offsets
        .clone() // doesn't impact performance, but uhhh this feels wrong??
        .into_iter()
        .map(|(d_x, d_y)| (x + d_x, y + d_y))
        .collect()
}

pub fn fast(path: &str, fast_type: Option<FastType>, threshold: Option<i16>) -> Result<Vec<Point>, ImageError> {
    let threshold:i16 = threshold.unwrap_or(DEFAULT_THRESHOLD);
    let fast_type:FastType = fast_type.unwrap_or(FastType::TYPE_9_16);

    let ctx:FastContext = fast_type.get_context();
    let indices_len = (ctx.fast.len() + ctx.slow.len()) as u8;
    let max_misses = indices_len - ctx.n;

    let mut fast_keypoint_matches:Vec<Point> = Vec::new();
    
    // load image as intensity map
    let img = image::open(path)?.to_luma();
    let (width, height) = img.dimensions();

    for y in ctx.radius..height-ctx.radius {
        'x_loop: for x in ctx.radius .. width-ctx.radius {
            let circle_pixels = get_circle_slice(&ctx, x as i32, y as i32)
                .into_iter()
                .map(|(p_x, p_y)| img.get_pixel(p_x as u32, p_y as u32).0[0] as i16)
                .collect::<Vec<i16>>();

            let center_pixel:i16 = img.get_pixel(x, y).0[0] as i16;

            //
            // get_pixel performs an index check on each call
            // these are wasted cycles since we guarantee (x, y) in bounds
            // since (x, y) is derived, and the image buffer is immutable
            // TODO: use unsafe variant or direct buffer addressing
            //

            let mut similars:u8 = 0;
            for index in 0..ctx.fast.len() {
                let diff:i16 = (circle_pixels[ctx.fast[index] as usize] - center_pixel).abs();
                if diff < threshold as i16 {
                    similars += 1;
                    if similars > 1 {
                        continue 'x_loop;
                    }
                }
            }

            for index in 0..ctx.slow.len() {
                let diff:i16 = (circle_pixels[ctx.slow[index] as usize] - center_pixel).abs();

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
    let color = [255, 0, 0];
    for (x, y) in vec {
        for (c_x, c_y) in get_circle_slice(&FastType::TYPE_9_16.get_context(), *x, *y) {
            img.get_pixel_mut(c_x as u32, c_y as u32).0 = color;
        }
    }
}
