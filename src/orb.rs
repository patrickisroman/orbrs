use bitvector::BitVector;
use cgmath::{prelude::*, Deg, Rad};
use image::{imageops::{blur, overlay}, Rgba};
use imageproc::drawing::draw_line_segment_mut;
use image::{
    DynamicImage, GenericImageView, GrayImage, ImageBuffer, ImageError, ImageFormat, RgbImage,
};
use std::cmp::{max, min};

use crate::{brief, common, fast};
use common::*;
use fast::FastKeypoint;

// Consts
const DEFAULT_BRIEF_LENGTH: usize = 256;

//
// Sobel Calculations
//

type SobelFilter = [[i32; 3]; 3];
const SOBEL_X: SobelFilter = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
const SOBEL_Y: SobelFilter = [[1, 2, 1], [0, 0, 0], [-1, -2, -1]];
const SOBEL_OFFSETS: [[(i32, i32); 3]; 3] = [
    [(-1, -1), (0, -1), (1, -1)],
    [(-1, 0), (0, 0), (1, 0)],
    [(-1, 1), (0, 1), (1, 1)],
];

unsafe fn sobel(img: &image::GrayImage, filter: &SobelFilter, x: i32, y: i32) -> u8 {
    let mut sobel: i32 = 0;
    for (i, row) in filter.iter().enumerate() {
        for (j, k) in row.iter().enumerate() {
            if *k == 0 {
                continue;
            }

            let offset = SOBEL_OFFSETS[i][j];
            let (x, y) = ((x + offset.0) as u32, (y + offset.1) as u32);
            let px = img.unsafe_get_pixel(x, y).0[0];
            sobel += px as i32 * *k;
        }
    }
    min(sobel.abs() as u8, u8::MAX)
}

fn create_sobel_image(img: &GrayImage) -> GrayImage {
    let mut new_image: GrayImage = ImageBuffer::new(img.width(), img.height());

    for y in 1..img.height() - 1 {
        for x in 1..img.width() - 1 {
            let mut px = new_image.get_pixel_mut(x, y);
            px.0[0] = unsafe { sobel(img, &SOBEL_Y, x as i32, y as i32) as u8 };
        }
    }

    new_image
}

//
// BRIEF Calculations
//

#[derive(Debug)]
pub struct Brief {
    pub x: i32,
    pub y: i32,
    b: BitVector,
}

impl Matchable for Brief {
    fn distance(&self, other: &Self) -> usize {
        (0..min(self.b.capacity(), other.b.capacity())).fold(0, |acc, x| {
            acc + (self.b.contains(x) != other.b.contains(x)) as usize
        })
    }
}

fn round_angle(angle: i32, increment: i32) -> i32 {
    let modulo: i32 = angle % increment;
    let complement: i32 = if angle < 0 {
        increment + modulo
    } else {
        increment - modulo
    };

    if modulo.abs() > (increment << 1) {
        return if angle < 0 {
            angle - complement
        } else {
            angle + complement
        };
    }

    angle - modulo
}

pub fn brief(
    blurred_img: &GrayImage,
    vec: &Vec<FastKeypoint>,
    brief_length: Option<usize>,
) -> Vec<Brief> {
    let brief_length = brief_length.unwrap_or(DEFAULT_BRIEF_LENGTH);
    let width: i32 = blurred_img.width() as i32;
    let height: i32 = blurred_img.height() as i32;

    // copy offsets into current frame on stack
    let offsets = brief::OFFSETS.clone();

    vec.into_iter()
        .map(|k| {
            let rotation = Deg::from(Rad(k.moment.rotation)).0.round() as i32;
            let rounded_angle = Deg(round_angle(rotation, 12) as f32);

            let cos_a = Deg::cos(rounded_angle);
            let sin_a = Deg::sin(rounded_angle);
            let (x, y) = k.location;

            let mut bit_vec = BitVector::new(brief_length);

            for (i, ((x0, y0), (x1, y1))) in offsets.iter().enumerate() {
                let mut steered_p1 = (
                    x + (x0 * cos_a - y0 * sin_a).round() as i32,
                    y + (x0 * sin_a + y0 * cos_a).round() as i32,
                );

                let mut steered_p2 = (
                    x + (x1 * cos_a - y1 * sin_a).round() as i32,
                    y + (x1 * sin_a + y1 * cos_a).round() as i32,
                );

                steered_p1.0 = max(min(steered_p1.0, width - 1), 0);
                steered_p2.0 = max(min(steered_p2.0, width - 1), 0);
                steered_p1.1 = max(min(steered_p1.1, height - 1), 0);
                steered_p2.1 = max(min(steered_p2.1, height - 1), 0);

                let brief_feature = blurred_img
                    .get_pixel(steered_p1.0 as u32, steered_p1.1 as u32)
                    .0[0]
                    > blurred_img
                        .get_pixel(steered_p2.0 as u32, steered_p2.1 as u32)
                        .0[0];

                if brief_feature {
                    bit_vec.insert(i);
                }
            }

            Brief {
                x: k.location.0,
                y: k.location.1,
                b: bit_vec,
            }
        })
        .collect::<Vec<Brief>>()
}

//
// ORB Calculations
//

pub fn orb(img: &DynamicImage, n: usize) -> Result<Vec<Brief>, ImageError> {
    let gray_img = img.to_luma();

    let mut keypoints: Vec<FastKeypoint> = fast::fast(&gray_img, None, None)?;

    let keypoints = adaptive_nonmax_suppression(&mut keypoints, n);

    let blurred_img = blur(&gray_img, 3.0);
    let brief_descriptors = brief(&blurred_img, &keypoints, None);

    Ok(brief_descriptors)
}

pub fn draw_orb(img1: &DynamicImage, img2: &DynamicImage, key_points1: &Vec<Brief>, key_points2: &Vec<Brief>, pair_indices: &Vec<(usize, usize)>) -> DynamicImage {
    let mut img = DynamicImage::new_rgb8(
        img1.width() + img2.width(),
        img1.height().max(img2.height()),
    );
    overlay(&mut img, img1, 0, 0);
    overlay(&mut img, img2, img1.width(), 0);
    pair_indices.iter().for_each(|(i1, i2)| {
        let start_point = (key_points1[*i1].x as f32, key_points1[*i1].y as f32);
        let end_point = (img1.width() as f32 + key_points2[*i2].x as f32, key_points2[*i2].y as f32);
        draw_line_segment_mut(
            &mut img,
            start_point,
            end_point,
            Rgba([0, 0, 0, 125]),
        );
    });
    img
}
