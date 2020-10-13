use image::{ImageError, GenericImageView, DynamicImage, ImageBuffer, Rgb};
use image::imageops::{resize, FilterType, blur};
use imageproc::drawing::draw_line_segment_mut;
use rand::{Rng, thread_rng};
// Move to rand_distr crate (rand::_::Normal is deprecated)
use rand::distributions::{Normal, Distribution};
use bitvector::BitVector;

use crate::fast;
use fast::Point;

#[derive(Debug)]
pub struct Moment {
    p: Point,
    x: u32,
    y: u32,
    rot: f64
}

#[derive(Debug)]
pub struct Brief {
    x: i32,
    y: i32,
    b: BitVector
}

const DEFAULT_BRIEF_LENGTH:usize = 128;

fn patch_moment(img: &image::GrayImage, x:u32, y:u32, x_moment:u32, y_moment:u32, moment_radius:Option<u32>) -> f32 {
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

fn moment_centroid(img: &image::GrayImage, x:u32, y:u32, moment_radius:Option<u32>) -> Moment {
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
        p: (x as i32, y as i32),
        x: mx.round() as u32,
        y: my.round() as u32,
        rot: y_diff.atan2(x_diff)
    }
}

fn brief(blurred_img: &image::GrayImage, vec: &Vec<Moment>, brief_length: Option<usize>, n: Option<usize>) -> Vec<Brief> {
    let brief_length = brief_length.unwrap_or(DEFAULT_BRIEF_LENGTH);
    let n = n.unwrap_or(5);

    // TODO FAST specifies two distributions are a bit more efficient than a single distribution
    let inner_dist = Normal::new(0.0, n as f64);
    let outer_dist = Normal::new(0.0, (n as f64)/2.0);

    let width:i32 = blurred_img.width() as i32;
    let height:i32 = blurred_img.height() as i32;

    vec.into_iter()
        .map(|m| {
            let mut bit_vec = BitVector::new(brief_length);

            for i in 0..bit_vec.capacity() {
                let mut p1 = (
                    m.p.0 + inner_dist.sample(&mut thread_rng()).round() as i32,
                    m.p.1 + inner_dist.sample(&mut thread_rng()).round() as i32
                );
                let mut p2 = (
                    m.p.0 + outer_dist.sample(&mut thread_rng()).round() as i32,
                    m.p.1 + outer_dist.sample(&mut thread_rng()).round() as i32
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
                x: m.p.0,
                y: m.p.1,
                b: bit_vec
            } 
        })
        .collect::<Vec<Brief>>()
}

// load image and pass around reference to image instead of loading from path
pub fn orb(path: &str) -> Result<Vec<Moment>, ImageError> {
    let keypoints = fast::fast(path, None, None)?;
    let img = image::open(path)?.to_luma();

    // create scale variances
    let (img1, img2, img3) = (
        resize(&img, img.width()/2, img.height()/2, FilterType::Gaussian),
        resize(&img, img.width()/4, img.height()/4, FilterType::Gaussian),
        resize(&img, img.width()/8, img.height()/8, FilterType::Gaussian)
    );

    let centroids = keypoints
        .into_iter()
        .map(|(x, y)| moment_centroid(&img, x as u32, y as u32, None))
        .collect::<Vec<Moment>>();

    let x = brief(&img, &centroids, None, Some(5));

    Ok(centroids)
}

pub fn draw_moments(img: &mut image::RgbImage, vec: &Vec<Moment>) {
    for moment in vec {
        draw_line_segment_mut(
            img,
            (moment.p.0 as f32, moment.p.1 as f32), 
            (moment.x as f32, moment.y as f32),
            Rgb([122, 122, 122])
        );

        img.get_pixel_mut(moment.x as u32, moment.y as u32).0 = [255, 255, 255];
        img.get_pixel_mut(moment.p.0 as u32, moment.p.1 as u32).0 = [0, 0, 0];
    }
}