use image::{ImageError, GenericImageView, DynamicImage, ImageBuffer};
use image::imageops::{resize, FilterType};
use crate::fast;

#[derive(Debug)]
struct Moment {
    x: u32,
    y: u32,
    rot: f64
}

fn moment_centroid(img: &image::GrayImage, x:u32, y:u32, moment_radius:Option<u32>) -> Moment {
    let p_m = patch_moment(img, x as u32, y as u32, 0, 0, moment_radius);
    let p_x = patch_moment(img, x as u32, y as u32, 1, 0, moment_radius);
    let p_y = patch_moment(img, x as u32, y as u32, 0, 1, moment_radius);

    Moment {
        x: (p_x/p_m),
        y: (p_y/p_m),
        rot: (p_x as f64).atan2(p_y as f64)
    }
}

fn patch_moment(img: &image::GrayImage, x:u32, y:u32, x_moment:u32, y_moment:u32, moment_radius:Option<u32>) -> u32 {
    let moment_radius = moment_radius.unwrap_or(3);

    // overflow is a possibility
    let mut patch_sum:u32 = 0;
    for mx in (x-moment_radius)..(x+moment_radius) {
        for my in (y-moment_radius)..(y+moment_radius) {
            patch_sum += mx.pow(x_moment) * my.pow(y_moment) * img.get_pixel(mx, my).0[0] as u32;
        }
    }

    patch_sum
}

// load image and pass around reference to image instead of loading from path
pub fn orb(path: &str) -> Result<bool, ImageError> {
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

    Ok(true)
}