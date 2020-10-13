#[allow(unused_imports)]

mod orb;
mod fast;

fn main() -> Result<(), image::ImageError> {
    let _ = test_orb()?;
    Ok(())
}

fn test_fast() -> Result<bool, image::ImageError> {
    let keypoints = fast::fast( "example/run.jpg", Some(fast::FastType::TYPE_9_16), None).unwrap();
    let mut img = image::open("example/run.jpg").unwrap().to_rgb();
    fast::draw_keypoints(&mut img, &keypoints);
    img.save_with_format("example/run.png", image::ImageFormat::Png)?;

    Ok(true)
}

fn test_orb() -> Result<bool, image::ImageError> {
    let keypoints = orb::orb("example/run.jpg")?;
    let mut img = image::open("example/run.jpg").unwrap().to_rgb();
    orb::draw_moments(&mut img, &keypoints);
    img.save_with_format("example/run.png", image::ImageFormat::Png)?;

    Ok(true)
}