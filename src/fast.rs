extern crate image;
use image::{GenericImageView, ImageError};

// Types
pub struct FastKeypoint {
    x: u32,
    y: u32,
    k: f32
}

pub type Point = (i32, i32);

// Consts
const DEFAULT_N:u8 = 12;
const DEFAULT_THRESHOLD:i16 = 45;
const CIRCLE_RADIUS:u32 = 3;
const CIRCLE_OFFSETS:[Point; 16] = [
    (0, -3), (1, -3),  (2, -2),  (3, -1),
    (3, 0),  (3, 1),   (2, 2),   (1, 3),
    (0, 3),  (-1, 3),  (-2, 2),  (-3, 1),
    (-3, 0), (-3, -1), (-2, -2), (-1, -3) 
];

const FAST_CHECK_INDICES:[u8; 4]  = [0, 4, 8, 12];
const SLOW_CHECK_INDICES:[u8; 12] = [1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15];

// Methods
fn get_circle_slice(x: i32, y:i32) -> Vec<Point> {
    CIRCLE_OFFSETS
        .to_vec()
        .into_iter()
        .map(|(d_x, d_y)| (x + d_x, y + d_y))
        .collect()
}

pub fn fast(path: &str, threshold: Option<i16>, n: Option<u8>) -> Result<Vec<Point>, ImageError> {
    let threshold:i16 = threshold.unwrap_or(DEFAULT_THRESHOLD);
    let n:u8 = n.unwrap_or(DEFAULT_N);

    assert!(CIRCLE_OFFSETS.len() as u8 >= n);
    let max_misses = CIRCLE_OFFSETS.len() as u8 - n;

    let mut fast_keypoint_matches:Vec<Point> = Vec::new();
    
    // load image as intensity map
    let img = image::open(path)?.to_luma();
    let (width, height) = img.dimensions();

    println!("{} x {} ", width, height);
    let start = std::time::Instant::now();
    for y in CIRCLE_RADIUS..height-CIRCLE_RADIUS {
        'x_loop: for x in CIRCLE_RADIUS .. width-CIRCLE_RADIUS {
            let circle_pixels = get_circle_slice(x as i32, y as i32)
                .into_iter()
                .map(|(p_x, p_y)| img.get_pixel(p_x as u32, p_y as u32).0[0] as i16)
                .collect::<Vec<i16>>();

            let center_pixel:i16 = img.get_pixel(x, y).0[0] as i16;

            let mut similars:u8 = 0;
            for index in 0..FAST_CHECK_INDICES.len() {
                let diff:i16 = (circle_pixels[FAST_CHECK_INDICES[index] as usize] - center_pixel).abs();
                if diff < threshold as i16 {
                    similars += 1;
                    if similars > 1 {
                        continue 'x_loop;
                    }
                }
            }

            for index in 0..SLOW_CHECK_INDICES.len() {
                let diff:i16 = (circle_pixels[SLOW_CHECK_INDICES[index] as usize] - center_pixel).abs();

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

    println!("{:?}", start.elapsed());

    Ok(fast_keypoint_matches)
}

pub fn draw_keypoints(img: &mut image::RgbImage, vec: &Vec<Point>) {
    let color = [255, 0, 0];
    for (x, y) in vec {
        for (c_x, c_y) in get_circle_slice(*x, *y) {
            img.get_pixel_mut(c_x as u32, c_y as u32).0 = color;
        }
    }
}
