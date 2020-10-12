# ORB (Oriented FAST and Rotated BRIEF) Keypoints with Rust

**FAST** Keypoints
![FAST Keypoints](example/fast.png)

Example running FAST and drawing keypoints:
```rust
use image;
use orbrs;

fn test() {
    let fast_keypoints = orbrs::fast::fast("example/test.jpg", fast::FastType::TYPE_9_16, None).unwrap();

    // draw the keypoints on the image
    let mut img = image::open("example/test.jpg").unwrap().to_rgb();
    orbrs::fast::draw_keypoints(&mut img, &fast_keypoints);
    img.save_with_format("example/fast_output.png", image::ImageFormat::Png);
}
```

Playing around with Rust. If you stumble across this, keep moving!

TODO:
- Switch to cargo lib (currently bin)
- Optimize FAST
- ORB