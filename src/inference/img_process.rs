use image::imageops::colorops::grayscale;
use image::{RgbImage, GrayImage, ImageBuffer};
use image::imageops::resize;
use imageproc::template_matching::{match_template, MatchTemplateMethod, find_extremes};
use log::{info, trace};

use crate::capture::RawImage;

#[inline]
fn get_index(width: u32, x: u32, y: u32) -> usize {
    (y * width + x) as usize
}

pub fn to_gray(raw: Vec<u8>, width: u32, height: u32) -> RawImage {
    let mut ans: Vec<f32> = vec![0.0; (width * height) as usize];
    for i in 0..width {
        for j in 0..height {
            let x = i;
            let y = height - j - 1;
            let b = raw[((y * width + x) * 4 + 0) as usize];
            let g = raw[((y * width + x) * 4 + 1) as usize];
            let r = raw[((y * width + x) * 4 + 2) as usize];

            let r = r as f32 / 255.0;
            let g = g as f32 / 255.0;
            let b = b as f32 / 255.0;

            let gray = r as f32 * 0.2989 + g as f32 * 0.5870 + b as f32 * 0.1140;
            let index = get_index(width, i, j);
            ans[index] = gray;
        }
    }

    RawImage {
        data: ans,
        h: height,
        w: width,
    }
}

pub fn normalize(im: &mut RawImage, auto_inverse: bool) {
    let width = im.w;
    let height = im.h;

    if width == 0 || height == 0 {
        return;
    }

    let data = &mut im.data;
    // info!("in normalize: width = {}, height = {}", width, height);

    let mut max: f32 = 0.0;
    let mut min: f32 = 256.0;

    for i in 0..width {
        for j in 0..height {
            let index = get_index(width, i, j);
            // info!("i = {}, j = {}, width = {}, index = {}", i, j, width, index);
            let p = data[index];
            if p > max {
                max = p;
            }
            if p < min {
                min = p;
            }
        }
    }

    let flag_pixel = data[get_index(width, width - 1, height - 1)];
    let flag_pixel = (flag_pixel - min) / (max - min);

    for i in 0..width {
        for j in 0..height {
            let index = get_index(width, i, j);
            let p = data[index];
            data[index] = (p - min) / (max - min);
            if auto_inverse && flag_pixel > 0.5 {
                // println!("123");
                data[index] = 1.0 - data[index];
            }
            // if data[index] < 0.6 {
            //     data[index] = 0.0;
            // }
        }
    }
}

pub fn crop(im: &RawImage) -> RawImage {
    let width = im.w;
    let height = im.h;

    let mut min_col = width - 1;
    let mut max_col = 0;
    let mut min_row = height - 1;
    let mut max_row = 0_u32;

    for i in 0..width {
        for j in 0..height {
            let index = get_index(width, i, j);
            let p = im.data[index];
            if p > 0.7 {
                if i < min_col {
                    min_col = i;
                }
                if i > max_col {
                    max_col = i;
                }
                break;
            }
        }
    }

    for j in 0..height {
        for i in 0..width {
            let index = get_index(width, i, j);
            let p = im.data[index];
            if p > 0.7 {
                if j < min_row {
                    min_row = j;
                }
                if j > max_row {
                    max_row = j;
                }
                break;
            }
        }
    }

    let new_height = max_row - min_row + 1;
    let new_width = max_col - min_col + 1;

    let mut ans: Vec<f32> = vec![0.0; (new_width * new_height) as usize];

    for i in min_col..=max_col {
        for j in min_row..=max_row {
            let index = get_index(width, i, j);
            let new_index = get_index(new_width, i - min_col, j - min_row);
            ans[new_index] = im.data[index];
        }
    }

    RawImage {
        data: ans,
        w: new_width,
        h: new_height,
    }
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
    trace!("res_x = {}, res_y = {}, res_val = {}", res_x, res_y, res_val);
    // info!("res_x = {}, res_y = {}, res_val = {}", res_x, res_y, res_val);
    if res_val < threshold {
        (res_x, res_y)
    } else {
        (-1, -1)
    }
}


// u8 sRGB to L channel
pub fn rgb_to_l(image: &RgbImage) -> GrayImage {
    let f = |t: f32| -> f32 {
        let thre: f32 = (6.*6.*6. /29./29./29.);
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
