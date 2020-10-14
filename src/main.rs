#[allow(unused_imports)]

mod orb;
mod fast;

fn main() {
    println!("{:?}", test_orb());
}

fn test_fast() -> Result<bool, image::ImageError> {
    let mut img = image::open("example/run.jpg")?;
    let mut gray_img = img.to_luma();
    let mut img = img.to_rgb();
    let keypoints = fast::fast(&gray_img, Some(fast::FastType::TYPE_9_16), None).unwrap();
    fast::draw_keypoints(&mut img, &keypoints);
    img.save_with_format("example/run.png", image::ImageFormat::Png)?;

    Ok(true)
}

fn test_orb() -> Result<bool, image::ImageError> {
    let mut img = image::open("example/run.jpg")?.to_luma();
    let keypoints = orb::orb(&img)?;
    //orb::draw_moments(&mut img, &keypoints);
    //img.save_with_format("example/run.png", image::ImageFormat::Png)?;

    Ok(true)
}