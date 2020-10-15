#[allow(unused_imports)]

mod orb;
mod fast;

fn main() {
    println!("{:?}", test_fast());
}

fn test_fast() -> Result<bool, image::ImageError> {
    let mut img = image::open("example/test.jpg")?;
    let mut gray_img = img.to_luma();
    let mut img = img.to_rgb();

    let mut keypoints = fast::fast(&gray_img, Some(fast::FastType::TYPE_9_16), None).unwrap();
    fast::calculate_fast_centroids(&gray_img, &mut keypoints);
    
    fast::draw_moments(&mut img, &keypoints);
    //orb::draw_moments(&mut img, &centroids);
    //fast::draw_keypoints(&mut img, &keypoints);
    img.save_with_format("example/test.png", image::ImageFormat::Png)?;

    Ok(true)
}