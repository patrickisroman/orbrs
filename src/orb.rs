use image::{ImageError, GenericImageView, DynamicImage, ImageBuffer, Rgb};
use image::imageops::{resize, FilterType, blur};
use imageproc::drawing::draw_line_segment_mut;
use rand::Rng;
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
    x: u32,
    y: u32,
    b: BitVector
}

fn patch_moment(img: &image::GrayImage, x:u32, y:u32, x_moment:u32, y_moment:u32, moment_radius:Option<u32>) -> u32 {
    let moment_radius = moment_radius.unwrap_or(3);

    if x < moment_radius || y < moment_radius {
        return 1;
    }

    let mut patch_sum:u32 = 0;
    for mx in (x-moment_radius)..(x+moment_radius+1) {
        for my in (y-moment_radius)..(y+moment_radius+1) {
            patch_sum += mx.pow(x_moment) * my.pow(y_moment) * img.get_pixel(mx, my).0[0] as u32;
        }
    }

    patch_sum
}

fn moment_centroid(img: &image::GrayImage, x:u32, y:u32, moment_radius:Option<u32>) -> Moment {
    let p_m = patch_moment(img, x as u32, y as u32, 0, 0, moment_radius);
    let p_x = patch_moment(img, x as u32, y as u32, 1, 0, moment_radius);
    let p_y = patch_moment(img, x as u32, y as u32, 0, 1, moment_radius);

    let (mx, my) = (
        (p_x/p_m),
        (p_y/p_m)
    );

    let x_diff = (x as i32 - mx as i32) as f64;
    let y_diff = (y as i32 - my as i32) as f64;

    Moment {
        p: (x as i32, y as i32),
        x: mx,
        y: my,
        rot: y_diff.atan2(x_diff)
    }
}

fn brief(blurred_img: &mut image::GrayImage, vec: &Vec<Moment>, n: i32) -> Vec<Brief> {
    let brief_len = 128;
    vec.into_iter()
        .map(|x| {
            Brief {
                x: 0,
                y: 0,
                b: BitVector::new(brief_len)
            } 
        })
        .collect::<Vec<Brief>>()
}

// load image and pass around reference to image instead of loading from path
pub fn orb(path: &str) -> Result<Vec<Moment>, ImageError> {
    let keypoints = fast::fast(path, None, None)?;
    let mut img = image::open(path)?.to_luma();

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