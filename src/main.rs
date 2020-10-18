#[allow(unused_imports)]

mod orb;
mod fast;

fn main() {
    println!("{:?}", test_fast());
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