mod orb;
mod fast;

fn main() {
    test();
}

fn test_fast() {
    let keypoints = fast::fast( "example/test.jpg", Some(fast::FastType::TYPE_9_16), None).unwrap();
    let mut img = image::open("example/test.jpg").unwrap().to_rgb();
    fast::draw_keypoints(&mut img, &keypoints);
    img.save_with_format("example/test.png", image::ImageFormat::Png);
}

fn test_orb() {
    let keypoints = orb::orb("example/test.jpg");
}