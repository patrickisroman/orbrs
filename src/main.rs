mod fast;

fn main() {
    
}

#[test]
fn test() {
    let keypoints = fast::fast("example/test.jpg", None, None).unwrap();
    let mut img = image::open("example/test.jpg").unwrap().to_rgb();
    fast::draw_keypoints(&mut img, &keypoints);
    img.save_with_format("example/test.png", image::ImageFormat::Png);
}