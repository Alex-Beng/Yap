use image::imageops::colorops::grayscale;
use image::{RgbImage, GrayImage, ImageBuffer};
use image::imageops::resize;
use imageproc::template_matching::{match_template, MatchTemplateMethod, find_extremes};
use imageproc::{self, contours};
use imageproc::contrast::adaptive_threshold;
use log::{info, trace};

use crate::capture::{RawImage, PixelRect};
use crate::info;

// 图像上的contour特征，用于contour matching
#[derive(Debug)]
pub struct ContourFeatures {
    pub contour: contours::Contour<u32>,
    pub contour_have_father: bool,
    pub bbox: PixelRect,
    pub bbox_wh_ratio: f32,
    pub area: u32,
    pub area_ratio: f32,
    pub bbox_area_avg_pixel: f32,
    pub contour_points_avg_pixel: f32,
    
    pub contour_len2_area_ratio: f32, // contour length^2 / area
    pub father_bbox_wh_ratio: f32,
    // pub father_contour_len2_area_ratio: f32, // father contour length^2 / area

    // 中心矩，平移不变及缩放不变（不同分辨率）
    // pub miu_11: f32,
    
    // TODO: Hu moments特征
}

#[inline]
pub fn contours_bbox(cont: contours::Contour<u32>) -> PixelRect {
    let mut left = u32::MAX;
    let mut top = u32::MAX;
    let mut right = 0;
    let mut bottom = 0;
    for point in cont.points {
        if point.x < left {
            left = point.x;
        }
        if point.x > right {
            right = point.x;
        }
        if point.y < top {
            top = point.y;
        }
        if point.y > bottom {
            bottom = point.y;
        }
    }
    PixelRect {
        left: left as i32,
        top: top as i32,
        width: (right - left) as i32,
        height: (bottom - top) as i32,
    }
}

impl ContourFeatures {
    pub fn new_empty() -> ContourFeatures {
        ContourFeatures {
            contour: contours::Contour {
                points: Vec::new(),
                border_type: contours::BorderType::Hole,
                parent: None,
            },
            contour_have_father: false,
            bbox: PixelRect {
                left: 0,
                top: 0,
                width: 0,
                height: 0,
            },
            bbox_wh_ratio: 0.0,
            area: 0,
            area_ratio: 0.0,
            bbox_area_avg_pixel: 0.0,
            contour_points_avg_pixel: 0.0,
            contour_len2_area_ratio: 0.0,
            father_bbox_wh_ratio: 0.0,
        }
    }

    pub fn new(
        contour: contours::Contour<u32>,
        father_contour: contours::Contour<u32>,
        contour_have_father: bool,
        full_image: &GrayImage,
    ) -> ContourFeatures {
        // copy contour
        let contour_clone = contours::Contour {
            points: contour.points.clone(),
            border_type: contour.border_type,
            parent: contour.parent,
        };
        let bbox = contours_bbox(contour);
        let bbox_wh_ratio = bbox.width as f32 / bbox.height as f32;
        let area = bbox.width * bbox.height;
        
        let image_size = full_image.dimensions();
        let area_ratio = area as f32 / (image_size.0 * image_size.1) as f32;
        let mut pixel_sum = 0;
        for i in bbox.left..(bbox.left + bbox.width) {
            for j in bbox.top..(bbox.top + bbox.height) {
                pixel_sum += full_image.get_pixel(i as u32, j as u32)[0] as u32;
            }
        }
        let bbox_area_avg_pixel = pixel_sum as f32 / area as f32;

        let mut pixel_sum = 0;
        let _points = &contour_clone.points;
        let contour_len = _points.len();
        for point in _points {
            pixel_sum += full_image.get_pixel(point.x, point.y)[0] as u32;
        }
        let contour_points_avg_pixel = pixel_sum as f32 / _points.len() as f32;
        let contour_len2_area_ratio = contour_len as f32 * contour_len as f32 / area as f32 / 20.;

        let father_bbox = contours_bbox(father_contour);
        let father_bbox_wh_ratio = father_bbox.width as f32 / father_bbox.height as f32;

        ContourFeatures {
            contour: contour_clone,
            contour_have_father,
            bbox,
            bbox_wh_ratio,
            area: area as u32,
            area_ratio,
            bbox_area_avg_pixel,
            contour_points_avg_pixel,
            contour_len2_area_ratio,
            father_bbox_wh_ratio,
        }
    }

    pub fn can_match(&self, other: &ContourFeatures,
        cosine_tolorance: f32,
     ) -> (f32, bool) {
        // let feat_vec1 = self.to_features_vec();
        // let feat_vec2 = other.to_features_vec();

        let cos_simi = cosine_similarity(&self.to_features_vec(), &other.to_features_vec());
        // println!("cos_simi = {}", cos_simi);
        if cos_simi < cosine_tolorance {
            return (cos_simi, false);
        }
        else {
            // info!("vec: {:?}", self.to_features_vec());
            // info!("vec: {:?}", other.to_features_vec());
            // info!("simi: {}", cos_simi);
            return (cos_simi, true);
        }
    }

    pub fn to_features_vec(&self) -> Vec<f32> {
        let mut ans = Vec::new();
        // ans.push(self.contour_have_father as u32 as f32);
        ans.push(self.bbox_wh_ratio / 0.7);
        ans.push(self.area_ratio / 0.010997644);
        ans.push(self.bbox_area_avg_pixel / 255.0 /  0.7003782);
        ans.push(self.contour_points_avg_pixel / 255.0 /  0.94793975);
        ans.push(self.contour_len2_area_ratio / 20.0 /  0.04316346);
        ans.push(self.father_bbox_wh_ratio / 1.21875);
        ans
    }
}


#[inline]
fn cosine_similarity(v1: &Vec<f32>, v2: &Vec<f32>) -> f32 {
    let dot_product = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum::<f32>();
    let v1_norm = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let v2_norm = v2.iter().map(|x| x * x).sum::<f32>().sqrt();
    let denominator = v1_norm * v2_norm;
    if denominator > 0.0 {
        dot_product / denominator
    } else {
        0.0
    }
}


#[inline]
fn get_index(width: u32, x: u32, y: u32) -> usize {
    (y * width + x) as usize
}


pub fn raw_to_img(im: &RawImage) -> GrayImage {
    let width = im.w;
    let height = im.h;
    let data = &im.data;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let index = get_index(width, x, y);
        let p = data[index];
        let pixel = (p * 255.0) as u32;
        let pixel: u8 = if pixel > 255 {
            255
        } else if pixel < 0 {
            0
        } else {
            pixel as u8
        };
        image::Luma([pixel])
    });

    img
}

pub fn uint8_raw_to_img(im: &RawImage) -> GrayImage {
    let width = im.w;
    let height = im.h;
    let data = &im.data;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let index = get_index(width, x, y);
        let pixel =  data[index] as u32;
        let pixel: u8 = if pixel > 255 {
            255
        } else if pixel < 0 {
            0
        } else {
            pixel as u8
        };
        image::Luma([pixel])
    });

    img
}


// template matching
// using SumOfSquaredErrorsNormalized
pub fn run_match_template(
    image: &GrayImage,
    template: &GrayImage,
    threshold: f32,
) -> (i32, i32) {
    // info!("{},{} {},{}", image.width(), image.height(), template.width(), template.height());
    let result = match_template(&image, &template, MatchTemplateMethod::SumOfSquaredErrorsNormalized);
    
    let res_mm = find_extremes(&result);
    let res_x = res_mm.min_value_location.0 as i32;
    let res_y = res_mm.min_value_location.1 as i32;
    let res_val = res_mm.min_value;
    // trace!("res_x = {}, res_y = {}, res_val = {}", res_x, res_y, res_val);
    // info!("res_x = {}, res_y = {}, res_val = {}", res_x, res_y, res_val);
    if res_val < threshold {
        (res_x, res_y)
    } else {
        (-1, -1)
    }
}


// use contours to speed up template matching
pub fn run_match_template_contours_speedup(
    image: &GrayImage,
    template: &GrayImage,
    threshold: f32,
) -> (i32, i32) {
    

    return (-1, -1)
}



// u8 sRGB to L channel
pub fn rgb_to_l(image: &RgbImage) -> GrayImage {
    let f = |t: f32| -> f32 {
        let thre: f32 = 6.*6.*6. /29./29./29.;
        let a: f32 = 1.0 / 3.0 * 29.0 * 29.0 / 6.0 / 6.0;
        let b: f32 = 16.0 / 116.0;
        
        if t > thre {
            t.powf(1.0/3.0)
        }
        else {
            a * t + b
        }
    };


    let width = image.width();
    let height = image.height();

    let mut ans = GrayImage::new(width, height);

    for i in 0..width {
        for j in 0..height {
            let pixel = image.get_pixel(i, j);
            let r = pixel[0] as f32 / 255.;
            let g = pixel[1] as f32 / 255.;
            let b = pixel[2] as f32 / 255.;

            // 因为白点Yn = 1, Y/Yn = Y 
            let y: f32 = 1.0 * r + 4.5906 * g + 0.0601 * b;
            let l = 116.0 * f(y) - 16.0;

            ans.put_pixel(i, j, image::Luma([l as u8]));
        }
    }
    ans
}
