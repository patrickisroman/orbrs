use image;
use orbrs;

fn main() {
    let mut img1 = image::open("examples/money1.jpg").unwrap();
    let mut img2 = image::open("examples/money2.jpg").unwrap();

    let n_keypoints = 50;

    let img1_keypoints = orbrs::orb::orb(&mut img1, n_keypoints).unwrap();
    let img2_keypoints = orbrs::orb::orb(&mut img2, n_keypoints).unwrap();

    let pair_indices = orbrs::common::match_indices(&img1_keypoints, &img2_keypoints);

    let img = orbrs::orb::draw_orb(&img1, &img2, &img1_keypoints, &img2_keypoints, &pair_indices);
    img.save_with_format("examples/money_output.png", image::ImageFormat::Png)
        .unwrap();
}
