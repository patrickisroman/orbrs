use image::{ImageError, GenericImageView, DynamicImage, ImageBuffer, Rgb, GrayImage, ImageFormat, RgbImage};
use image::imageops::{resize, FilterType, blur, filter3x3};
use rand::{Rng, thread_rng};
// Move to rand_distr crate (rand::_::Normal is deprecated)
use rand::distributions::{Normal, Distribution};
use bitvector::BitVector;

use crate::fast;
use fast::{Point, FastKeypoint, Moment, };

const DEFAULT_BRIEF_LENGTH:usize = 128;

//
// Sobel Calculations
//

type SobelFilter = [[i32;3];3];
const SOBEL_X : SobelFilter = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
const SOBEL_Y : SobelFilter = [[1, 2, 1], [0, 0, 0], [-1, -2, -1]];
const SOBEL_OFFSETS : [[(i32, i32);3];3] = [[(-1, -1), (0, -1), (1, -1)], [(-1, 0),  (0, 0), (1, 0)], [(-1, 1), (0, 1), (1, 1)]];

// can be more efficient if we consider 3x1 slices of the array
unsafe fn sobel(img: &image::GrayImage, filter: &SobelFilter, x: i32, y: i32) -> i32 {
    let mut sobel:i32 = 0;
    for (i, row) in filter.iter().enumerate() {
        for (j, k) in row.iter().enumerate() {
            if *k == 0 { continue }

            let offset = SOBEL_OFFSETS[i][j];
            let (x, y) = ((x + offset.0) as u32, (y + offset.1) as u32);
            let px = img.get_pixel(x, y).0[0];
            sobel += px as i32 * *k;
        }
    }
    std::cmp::min(sobel.abs(), 255)
}

fn create_sobel_image(img: &GrayImage) -> GrayImage {
    let mut new_image:GrayImage = ImageBuffer::new(img.width(), img.height());

    for y in 1..img.height()-1 {
        for x in 1..img.width()-1 {
            let mut px = new_image.get_pixel_mut(x, y);
            px.0[0] = unsafe { sobel(img, &SOBEL_Y, x as i32, y as i32) as u8 };
        }
    }

    new_image
}

//
// BRIEF Calculations
//

#[derive(Debug)]
pub struct Brief {
    x: i32,
    y: i32,
    b: BitVector
}

fn brief_distance(brief1: &Brief, brief2: &Brief) -> usize {
    brief1.b.intersection(&brief2.b).len()
}

fn brief(blurred_img: &GrayImage, vec: &Vec<FastKeypoint>, brief_length: Option<usize>, n: Option<usize>) -> Vec<Brief> {
    let brief_length = brief_length.unwrap_or(DEFAULT_BRIEF_LENGTH);
    let n = n.unwrap_or(5);

    // TODO FAST specifies two distributions are a bit more efficient than a single distribution
    let inner_dist = Normal::new(0.0, n as f64);
    let outer_dist = Normal::new(0.0, (n as f64)/2.0);

    let width:i32 = blurred_img.width() as i32;
    let height:i32 = blurred_img.height() as i32;

    vec.into_iter()
        .map(|k| {
            let mut bit_vec = BitVector::new(brief_length);

            for i in 0..bit_vec.capacity() {
                let mut p1 = (
                    k.location.0 + inner_dist.sample(&mut thread_rng()).round() as i32,
                    k.location.1 + inner_dist.sample(&mut thread_rng()).round() as i32
                );
                let mut p2 = (
                    k.location.0 + outer_dist.sample(&mut thread_rng()).round() as i32,
                    k.location.1 + outer_dist.sample(&mut thread_rng()).round() as i32
                );

                p1.0 = std::cmp::max(std::cmp::min(p1.0, width - 1), 0);
                p2.0 = std::cmp::max(std::cmp::min(p2.0, width - 1), 0);
                p1.1 = std::cmp::max(std::cmp::min(p1.1, height - 1), 0);
                p2.1 = std::cmp::max(std::cmp::min(p2.1, height - 1), 0);

                let brief_feature = blurred_img.get_pixel(p1.0 as u32, p1.1 as u32).0[0] >
                                    blurred_img.get_pixel(p2.0 as u32, p2.1 as u32).0[0];

                if brief_feature {
                    bit_vec.insert(i);
                }
            }

            Brief {
                x: k.location.0,
                y: k.location.1,
                b: bit_vec
            } 
        })
        .collect::<Vec<Brief>>()
}

//
// ORB Calculations
//

// load image and pass around reference to image instead of loading from path
pub fn orb(img: &GrayImage) -> Result<Vec<FastKeypoint>, ImageError> {
    // let keypoints = fast::fast(img, None, None)?;

    // let centroids = keypoints
    //     .into_iter()
    //     .map(|k| moment_centroid(&img, x as u32, y as u32, None))
    //     .collect::<Vec<Moment>>();

    // let blurred_img = blur(img, 2.5);
    // let brief_descriptors = brief(&blurred_img, &centroids, None, None);
    // Ok(centroids)
    
    let centroids:Vec<FastKeypoint> = vec![];
    Ok(centroids)
}