#[allow(unused_imports)]

mod orb;
mod fast;

use image::{ImageError, GenericImageView, DynamicImage, ImageBuffer, Rgb, GrayImage, ImageFormat, RgbImage};
use imageproc::drawing::draw_line_segment_mut;

use bitvector::BitVector;

fn main() {
    println!("{:?}", test_orb_mapping("example/a.png", "example/b.png"));
}

fn test_orb_mapping(img1: &str, img2: &str) -> Result<bool, image::ImageError> {
    let mut img1 = image::open(img1)?;
    let mut img2 = image::open(img2)?;

    let n = 10;

    let mut key1 = orb::orb(&img1.to_luma(), n)?;
    let mut key2 = orb::orb(&img2.to_luma(), n)?;

    for k in &key1 {
        println!("{:?}", k);
    }

    println!("\n");
    for k in &key2 {
        println!("{:?}", k);
    }

    let mut outimg = image::open("example/out.png")?.to_rgb();
    let comp = orb::match_brief(&key1, &key2);

    for (i, j) in &comp {
        let k1 = key1.get(*i).unwrap();
        let k2 = key2.get(*j).unwrap();

        draw_line_segment_mut(
            &mut outimg,
            (k1.x as f32, k1.y as f32),
            (600.0 + k2.x as f32, k2.y as f32),
            Rgb([255, 255, 255])
        );
    }

    outimg.save_with_format("out.png", image::ImageFormat::Png);

    Ok(true)
}

fn test_fast() -> Result<bool, image::ImageError> {
    let mut img = image::open("example/money1.jpg")?;
    let mut gray_img = img.to_luma();
    let mut img = img.to_rgb();

    let mut keypoints = fast::fast(&gray_img, Some(fast::FastType::TYPE_9_16), None).unwrap();

    let mut supp = orb::adaptive_nonmax_suppression(&mut keypoints, 200);

    fast::calculate_fast_centroids(&gray_img, &mut supp);
    fast::draw_moments(&mut img, &supp);
    img.save_with_format("example/money1.png", image::ImageFormat::Png)?;

    Ok(true)
}

fn test_orb() -> Result<bool, image::ImageError> {
    let mut img = image::open("example/test.jpg")?;
    let mut gray_img = img.to_luma();
    let mut img = img.to_rgb();

    let mut keypoints = fast::fast(&gray_img, Some(fast::FastType::TYPE_9_16), None).unwrap();
    fast::calculate_fast_centroids(&gray_img, &mut keypoints);

    let briefs = orb::brief(&gray_img, &keypoints, None, None);

    for brief in &briefs {
        if brief.b.len() == 0 {
            img.get_pixel_mut(brief.x as u32, brief.y as u32).0 = [255, 0, 0];
        }
    }

    img.save_with_format("out.png", image::ImageFormat::Png)?;

    Ok(true)
}