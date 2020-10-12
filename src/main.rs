#[allow(unused_imports)]

mod orb;
mod fast;

fn main() {
    test_fast();
}

fn test_fast() {
    let keypoints = fast::fast( "example/run.jpg", Some(fast::FastType::TYPE_9_16), None).unwrap();
    let mut img = image::open("example/run.jpg").unwrap().to_rgb();
    fast::draw_keypoints(&mut img, &keypoints);
    img.save_with_format("example/run.png", image::ImageFormat::Png);
}

fn test_orb() {
    let keypoints = orb::orb("example/test.jpg");
}