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
pub struct ContourFeatures {
    pub contour: contours::Contour<u32>,
    pub contour_have_father: bool,
    pub bbox: PixelRect,
    pub bbox_wh_ratio: f32,
    pub area: u32,
    pub area_ratio: f32,
    pub bbox_area_avg_pixel: f32,
    // TODO: 边界点 pixel avg特征
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
        }
    }

    pub fn new(
        contour: contours::Contour<u32>,
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

        ContourFeatures {
            contour: contour_clone,
            contour_have_father,
            bbox,
            bbox_wh_ratio,
            area: area as u32,
            area_ratio,
            bbox_area_avg_pixel,
        }
    }

    pub fn can_match(&self, other: &ContourFeatures,
        tolorance_bbox_wh_ratio: f32,
        tolorance_area_ratio: f32,
        tolorance_bbox_area_avg_pixel: f32,
     ) -> bool {
        // info!("self.contour_have_father = {}, other.contour_have_father = {}", self.contour_have_father, other.contour_have_father)
        // info!("self.bbox_wh_ratio = {}, other.bbox_wh_ratio = {}", self.bbox_wh_ratio, other.bbox_wh_ratio);
        // info!("self.area_ratio = {}, other.area_ratio = {}", self.area_ratio, other.area_ratio);
        // info!("self.bbox_area_avg_pixel = {}, other.bbox_area_avg_pixel = {}", self.bbox_area_avg_pixel, other.bbox_area_avg_pixel);
        if self.contour_have_father != other.contour_have_father {
            return false;
        }
        else if (self.bbox_wh_ratio - other.bbox_wh_ratio).abs() > tolorance_bbox_wh_ratio {
            return false;
        }
        else if (self.area_ratio - other.area_ratio).abs() > tolorance_area_ratio {
            return false;
        }
        else if (self.bbox_area_avg_pixel - other.bbox_area_avg_pixel).abs() > tolorance_bbox_area_avg_pixel {
            return false;
        }
        // TODO: Hu moments
        else {
            return true;
        }
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
