use std::cmp::{min, max};
use image::{ImageError, GenericImageView, DynamicImage, ImageBuffer, Rgb, GrayImage, ImageFormat, RgbImage};
use image::imageops::{blur};
use cgmath::{prelude::{*},Rad, Deg};
use bitvector::BitVector;

use crate::{fast, brief};
use fast::{FastKeypoint};

// Consts
const DEFAULT_BRIEF_LENGTH:usize = 256;

//
// Sobel Calculations
//

type SobelFilter = [[i32;3];3];
const SOBEL_X : SobelFilter = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
const SOBEL_Y : SobelFilter = [[1, 2, 1], [0, 0, 0], [-1, -2, -1]];
const SOBEL_OFFSETS : [[(i32, i32);3];3] = [[(-1, -1), (0, -1), (1, -1)], [(-1, 0),  (0, 0), (1, 0)], [(-1, 1), (0, 1), (1, 1)]];

unsafe fn sobel(img: &image::GrayImage, filter: &SobelFilter, x: i32, y: i32) -> u8 {
    let mut sobel:i32 = 0;
    for (i, row) in filter.iter().enumerate() {
        for (j, k) in row.iter().enumerate() {
            if *k == 0 { continue }

            let offset = SOBEL_OFFSETS[i][j];
            let (x, y) = ((x + offset.0) as u32, (y + offset.1) as u32);
            let px = img.unsafe_get_pixel(x, y).0[0];
            sobel += px as i32 * *k;
        }
    }
    min(sobel.abs() as u8, u8::MAX)
}

fn create_sobel_image(img: &GrayImage) -> GrayImage {
    let mut new_image:GrayImage = ImageBuffer::new(img.width(), img.height());

    for y in 1..img.height()-1 {
        for x in 1..img.width()-1 {
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
    b: BitVector
}

impl Brief {
    pub fn hamming_dist(&self, other: &Brief) -> usize {
        (0..min(self.b.capacity(), other.b.capacity()))
        .fold(0, |acc, x| {
            acc + (self.b.contains(x) != other.b.contains(x)) as usize
        })
    }
}

fn round_angle(angle: i32, increment: i32) -> i32 {
    let modulo:i32 = angle % increment;
    let complement:i32 = if angle < 0 
        { increment + modulo } else { increment - modulo} ;

    if modulo.abs() > (increment << 1) {
        return if angle < 0 { angle - complement } else { angle + complement };
    }

    angle - modulo
}

pub fn adaptive_nonmax_suppression(vec: &mut Vec<FastKeypoint>, n: usize) -> Vec<FastKeypoint> {
    let mut maximal_keypoints = vec![];
    for i in 1..vec.len() - 1 {
        let d1 = vec[i];
        let mut min_dist:f32 = f32::MAX;
        for j in 0..i {
            let d0 = vec[j];
            let dist = d0.dist(&d1);
            if dist < min_dist {
                min_dist = dist;
            }
        }
        vec[i].nms_dist = min_dist;
    }

    vec.sort_by(|a, b| b.nms_dist.partial_cmp(&a.nms_dist).unwrap());

    for k in 0..n {
        maximal_keypoints.push(vec[k]);
    }
    maximal_keypoints
}

pub fn brief(blurred_img: &GrayImage, vec: &Vec<FastKeypoint>, brief_length: Option<usize>) -> Vec<Brief> {
    let brief_length = brief_length.unwrap_or(DEFAULT_BRIEF_LENGTH);
    let width:i32 = blurred_img.width() as i32;
    let height:i32 = blurred_img.height() as i32;

    vec.into_iter()
        .map(|k| {
            let rotation = Deg::from(Rad(k.moment.rotation)).0.round() as i32;
            let rounded_angle = Deg(round_angle(rotation, 12) as f32);

            let cos_a = Deg::cos(rounded_angle);
            let sin_a = Deg::sin(rounded_angle);
            let (x, y) = k.location;

            let mut bit_vec = BitVector::new(brief_length);

            for i in 0..brief_length {
                let ((x0, y0), (x1, y1)) = brief::OFFSETS[i];

                let mut steered_p1 = (
                    x + (x0 * cos_a - y0 * sin_a).round() as i32,
                    y + (x0 * sin_a + y0 * cos_a).round() as i32
                );

                let mut steered_p2 = (
                    x + (x1 * cos_a - y1 * sin_a).round() as i32,
                    y + (x1 * sin_a + y1 * cos_a).round() as i32
                );

                steered_p1.0 = max(min(steered_p1.0, width - 1), 0);
                steered_p2.0 = max(min(steered_p2.0, width - 1), 0);
                steered_p1.1 = max(min(steered_p1.1, height - 1), 0);
                steered_p2.1 = max(min(steered_p2.1, height - 1), 0);

                let brief_feature = blurred_img.get_pixel(steered_p1.0 as u32, steered_p1.1 as u32).0[0] >
                                    blurred_img.get_pixel(steered_p2.0 as u32, steered_p2.1 as u32).0[0];

                if brief_feature {
                    bit_vec.insert(i);
                }
            }

            Brief {
                x: k.location.0,
                y: k.location.1,
                b: bit_vec
            } 
        })
        .collect::<Vec<Brief>>()
}

//
// ORB Calculations
//

pub fn orb(img: &DynamicImage, n:usize) -> Result<Vec<Brief>, ImageError> {
    let gray_img = img.to_luma();
    
    let mut keypoints = fast::fast(&gray_img, None, None)?;

    let keypoints = adaptive_nonmax_suppression(&mut keypoints, n);

    let blurred_img = blur(&gray_img, 3.0);
    let brief_descriptors = brief(&blurred_img, &keypoints, None);

    Ok(brief_descriptors)
}

pub fn match_brief(vec1: &Vec<Brief>, vec2: &Vec<Brief>) -> Vec<(usize, usize)>{
    assert_eq!(vec1.len(), vec2.len());

    let mut index_vec = vec![];
    let len = min(vec1.len(), vec2.len());
    let mut matched_indices = BitVector::new(len);

    for i in 0..len {
        let mut min_hamming_dist:usize = usize::MAX;
        let mut matched_index:usize = 0;
        for j in 0..vec2.len() {
            if matched_indices.contains(j) { 
                continue 
            }

           let dist = vec1[i].hamming_dist(&vec2[j]);
           if dist < min_hamming_dist {
               min_hamming_dist = dist;
               matched_index = j;
           }
        }

        index_vec.push((i as usize, matched_index as usize));
        matched_indices.insert(matched_index);
    }

    index_vec
}