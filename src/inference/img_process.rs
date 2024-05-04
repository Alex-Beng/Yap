use image::{RgbImage, GrayImage, ImageBuffer};
use imageproc::template_matching::{match_template, MatchTemplateMethod, find_extremes};
use imageproc::{self, contours};
use imageproc::contrast::adaptive_threshold;
use log::info;

use crate::capture::{RawImage, PixelRect};

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

    pub fn new_tp(
        contour: contours::Contour<u32>,
        full_image: &GrayImage,
    ) -> ContourFeatures {
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

        // 计算bbox内的平均像素值
        let mut pixel_sum = 0;
        for i in bbox.left..(bbox.left + bbox.width) {
            for j in bbox.top..(bbox.top + bbox.height) {
                pixel_sum += full_image.get_pixel(i as u32, j as u32)[0] as u32;
            }
        }
        let bbox_area_avg_pixel = pixel_sum as f32 / area as f32;

        // 计算contour内的平均像素值
        let mut pixel_sum = 0;
        let _points = &contour_clone.points;
        let contour_len = _points.len();
        for point in _points {
            pixel_sum += full_image.get_pixel(point.x, point.y)[0] as u32;
        }
        let contour_points_avg_pixel = pixel_sum as f32 / _points.len() as f32;

        let contour_len2_area_ratio = contour_len as f32 * contour_len as f32 / area as f32 / 30.;

        let mut feat = ContourFeatures::new_empty();
        feat.contour = contour_clone;
        feat.bbox = bbox;
        feat.bbox_wh_ratio = bbox_wh_ratio;
        feat.area = area as u32;
        feat.area_ratio = area_ratio;
        feat.bbox_area_avg_pixel = bbox_area_avg_pixel;
        feat.contour_points_avg_pixel = contour_points_avg_pixel;
        feat.contour_len2_area_ratio = contour_len2_area_ratio;
        feat
    }

    pub fn can_match(&self, other: &[f32],
        cosine_tolorance: f32,
     ) -> (f32, bool) {
        // let feat_vec1 = self.to_features_vec();
        // let feat_vec2 = other.to_features_vec();

        let cos_simi = cosine_similarity(&self.to_features_vec(), other);
        // println!("cos_simi = {}", cos_simi);
        if cos_simi < cosine_tolorance {
            (cos_simi, false)
        }
        else {
            // info!("vec: {:?}", self.to_features_vec());
            // info!("vec: {:?}", other.to_features_vec());
            // info!("simi: {}", cos_simi);
            (cos_simi, true)
        }
    }

    pub fn can_match_tp(&self, other: &[f32],
        cosine_tolorance: f32,
     ) -> (f32, bool) {
        // let feat_vec1 = self.to_feature_vec_tp();
        // let feat_vec2 = other;

        let cos_simi = cosine_similarity(&self.to_feature_vec_tp(), other);
        // println!("cos_simi = {}", cos_simi);
        if cos_simi < cosine_tolorance {
            (cos_simi, false)
        }
        else {
            // info!("vec: {:?}", self.to_feature_vec_tp());
            // info!("vec: {:?}", other);
            // info!("simi: {}", cos_simi);
            (cos_simi, true)
        }
    }

    // 累了，居然硬编码的F key的特征
    // 就这样吧
    pub fn to_features_vec(&self) -> Vec<f32> {
        vec![
            // self.contour_have_father as u32 as f32
            // self.bbox_wh_ratio / 0.7
            // self.area_ratio / 0.010997644
            self.bbox_area_avg_pixel / 255.0 /  0.7003782,
            self.contour_points_avg_pixel / 255.0 /  0.94793975,
            // self.contour_len2_area_ratio / 20.0 /  0.04316346
            self.father_bbox_wh_ratio / 1.21875
        ]
    }

    // tp的to vec
    pub fn to_feature_vec_tp(&self) -> Vec<f32> {
        vec![
            self.bbox_wh_ratio / 7.4126983,
            self.area_ratio / 0.81895614,
            self.bbox_area_avg_pixel / 255.0 / 0.348_715_84, // clippy said 0.348715843137255 over float literal
            self.contour_points_avg_pixel / 255.0 / 0.513_257_6,
            self.contour_len2_area_ratio / 1.1059493,
        ]
    }
}


#[inline]
fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
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

    ImageBuffer::from_fn(width, height, |x, y| {
        let index = get_index(width, x, y);
        let p = data[index];
        let pixel = (p * 255.0) as u32;
        let pixel: u8 = if pixel > 255 {
            255
        } else {
            pixel as u8
        };
        image::Luma([pixel])
    })
}

pub fn uint8_raw_to_img(im: &RawImage) -> GrayImage {
    let width = im.w;
    let height = im.h;
    let data = &im.data;

    ImageBuffer::from_fn(width, height, |x, y| {
        let index = get_index(width, x, y);
        let pixel =  data[index] as u32;
        let pixel: u8 = if pixel > 255 {
            255
        } else {
            pixel as u8
        };
        image::Luma([pixel])
    })
}


// template matching
// using SumOfSquaredErrorsNormalized
pub fn run_match_template(
    image: &GrayImage,
    template: &GrayImage,
    threshold: f32,
) -> (i32, i32) {
    // info!("{},{} {},{}", image.width(), image.height(), template.width(), template.height());
    let result = match_template(image, template, MatchTemplateMethod::SumOfSquaredErrorsNormalized);
    
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
    // in order to reduce clippy warning
    let _ = image;
    let _ = template;
    let _ = threshold;
    
    (-1, -1)
}


pub fn run_contours_cosine_matching(
    image: &GrayImage,
    template_cosine: &[f32],
    threshold: f32,
) -> (i32, i32) {
    let f_area_cap_gray = image;
    let f_area_cap_thre = adaptive_threshold(f_area_cap_gray, 14);
    let f_area_contours: Vec<contours::Contour<u32>> = imageproc::contours::find_contours(&f_area_cap_thre);
    
    let mut f_cnt = 0;
    let mut rel_x = -1;
    let mut rel_y = -1;

    let mut best_match = 0.;
    // let mut best_father_bbox = PixelRect {
    //     left: 0,
    //     top: 0,
    //     width: 0,
    //     height: 0,
    // };

    // for contour in f_area_contours {
    for i in 0..f_area_contours.len() {
        let contour = &f_area_contours[i];
        let contour_clone = imageproc::contours::Contour {
            points: contour.points.clone(),
            border_type: contour.border_type,
            parent: contour.parent,
        };
        let has_parent = contour.parent.is_some();
        if !has_parent { continue; }

        let father_contour = &f_area_contours[contour.parent.unwrap()];
        let father_contour_clone = imageproc::contours::Contour {
            points: father_contour.points.clone(),
            border_type: father_contour.border_type,
            parent: father_contour.parent,
        };

        let contour_feat = ContourFeatures::new(
            contour_clone,
            father_contour_clone,
            has_parent,
            f_area_cap_gray
        );
        
        let (cos_simi, _valid) = contour_feat.can_match(template_cosine, threshold);
        if contour_feat.contour_have_father && _valid {
            f_cnt += 1;

            // compute the rel x and y
            
            // 使用父亲轮廓bbox的计算
            let father_contour = &f_area_contours[contour.parent.unwrap()];
            let father_contour = imageproc::contours::Contour {
                points: father_contour.points.clone(),
                border_type: father_contour.border_type,
                parent: father_contour.parent,
            };
            let father_contour_bbox = contours_bbox(father_contour);

            rel_y = father_contour_bbox.top;
            rel_x = father_contour_bbox.left;
            // rel_y = bbox.top - self.config.info.f_area_position.top;
            // rel_x = 1;
            if cos_simi > best_match {
                best_match = cos_simi;
                // best_father_bbox = father_contour_bbox;
            }
        }
    }
    // f_cnt
    info!("f_cnt = {}", f_cnt);
    (rel_x, rel_y)
}


pub fn run_alpha_triangle_matching(
    image: &GrayImage,
    offset: u32,
) -> (i32, i32) {
    let f_area_cap_gray = image;
    let f_area_cap_thre = adaptive_threshold(f_area_cap_gray, 14);
    let f_area_contours: Vec<contours::Contour<u32>> = imageproc::contours::find_contours(&f_area_cap_thre);
    
    let mut f_cnt = 0;
    let mut rel_x = -1;
    let mut rel_y = -1;

    let mut max_area = 0;


    for contour in f_area_contours {
        // 小三角没有父母
        if contour.parent.is_some() { continue; }

        let bbox = contours_bbox(contour);

        // 小三角的宽高比约为 1:2
        let cont_wh_ratio = bbox.width as f32 / bbox.height as f32;
        if (cont_wh_ratio-0.5).abs() > 0.1 { continue; }

        // 算了，直接视为有效

        f_cnt += 1;

        if bbox.width * bbox.height > max_area {
            max_area = bbox.width * bbox.height;
            rel_x = bbox.left;
            rel_y = bbox.top;
        }    
    }
    info!("f_cnt = {}", f_cnt);

    (rel_x, rel_y - offset as i32)
}

// 使用最初版本的直接统计的方法
pub fn run_naive_alpha_triangle_matching(
    image: &GrayImage,
    offset: u32,
) -> (i32, i32) {
    // 实际上就是查找所有像素点不为0的坐标均值
    let mut x_sum = 0;
    let mut y_sum = 0;
    let mut cnt = 0;
    let mut pixel_sum = 0;
    for i in 0..image.width() {
        for j in 0..image.height() {
            let pixel = image.get_pixel(i, j)[0];
            if pixel != 0 {
                x_sum += i;
                y_sum += j;
                cnt += 1;
                pixel_sum += pixel as u32;
            }
        }
    }
    if cnt == 0 {
        return (-1, -1);
    }
    let x_avg = x_sum / cnt;
    let y_avg = y_sum / cnt;
    let pixel_avg = pixel_sum as f32 / cnt as f32;

    info!("pixel_avg = {}", pixel_avg);
    // 需要减掉offset，即text高度的一半
    let y_avg = y_avg - offset;

    (x_avg as i32, y_avg as i32)
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
