extern crate image;
use image::{ImageError, GenericImageView, DynamicImage, ImageBuffer};
use image::imageops::{resize, FilterType};
use crate::fast;

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

    let k1 = vec![(0, 0)];
    let k2 = vec![(0, 0)];
    let k3 = vec![(0, 0)];
    
    Ok(true)
}