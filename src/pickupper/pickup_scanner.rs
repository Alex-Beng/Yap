use std::collections::HashSet;
use std::time::SystemTime;

use crate::common::sleep;
use crate::inference::img_process::{run_match_template, pre_process};
use crate::{info::PickupInfo, common};
use crate::inference::inference::CRNNModel;
use crate::capture::{RawCaptureImage, self, PixelRect, RawImage};

use image::imageops::{grayscale, self};
use image::{GrayImage, ImageBuffer, Luma, ColorType, GenericImage};
use imageproc::definitions::Image;
use tract_onnx::prelude::*;
use serde_json;
use enigo::*;
use log::{info, warn};

pub struct PickupScanner {
    model: CRNNModel,
    enigo: Enigo,

    info: PickupInfo,

    f_template: GrayImage,
    
    // 黑名单
    black_list: HashSet<String>,
    // 所有物品
    all_list: HashSet<String>
}

impl PickupScanner {
    pub fn new(info: PickupInfo, black_list_path: String) -> PickupScanner {
        let mut bk_list: HashSet<String> = HashSet::new();
        // println!("black list path: {}", black_list_path);
        // 从black_list_path读取json中每一个String，加入到bk_list中
        // 老子也硬编码得了
        let black_list_path = "./black_lists.json";
        let mut file = File::open(black_list_path).expect("Failed to open black list file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read black list file");

        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let bk_items = json.as_array().unwrap();
        for item in bk_items {
            bk_list.insert(item.as_str().unwrap().to_string());
            info!("black list: {}", item.as_str().unwrap().to_string());
            
        }

        let mut all_list: HashSet<String> = HashSet::new();
        let content = String::from(include_str!("../../models/all_list.json"));
        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let al_items = json.as_array().unwrap();
        for item in al_items {
            all_list.insert(item.as_str().unwrap().to_string());
        }


        let template =image::open("models/FFF.bmp".to_string()).unwrap();
        let template = grayscale(&template);

        PickupScanner {
            model: CRNNModel::new(String::from("model_training.onnx"), String::from("index_2_word.json")),
            enigo: Enigo::new(),

            info,

            f_template: template,

            black_list: bk_list,
            all_list: all_list,
        }
    }

    fn capture_f_area(&mut self) -> Result<image::RgbImage, String> {
        let now = SystemTime::now();
        let w = self.info.f_area_position.right - self.info.f_area_position.left;
        let h = self.info.f_area_position.bottom - self.info.f_area_position.top;
        let rect: PixelRect = PixelRect {
            left: self.info.left as i32 + self.info.f_area_position.left,
            top: self.info.top as i32 + self.info.f_area_position.top,
            width: w,
            height: h,
        };

        let img = capture::capture_absolute_image(&rect)?;
        // info!("capture time: {}ms", now.elapsed().unwrap().as_millis());
        
        Ok(img)
    }

    fn capture_f_text(&mut self, rel_y: i32) -> Result<image::RgbImage, String> {
        let w = self.info.pickup_x_end - self.info.pickup_x_beg;
        let h = self.f_template.height();
        let rect: PixelRect = PixelRect {
            left: self.info.left as i32 + self.info.pickup_x_beg as i32, 
            top: self.info.top as i32 + self.info.f_area_position.top + rel_y,
            width: w as i32,
            height: h as i32,
        };
        let img = capture::capture_absolute_image(&rect)?;
        // info!("capture time: {}ms", now.elapsed().unwrap().as_millis());
        Ok(img)
    }

    

    pub fn start(&mut self, c: i32) {
        let mut cnt = c;
        loop {
            sleep(100);
            let f_area_cap = self.capture_f_area().unwrap();
            let f_area_cap_gray = grayscale(&f_area_cap);
            f_area_cap_gray.save("farea.jpg").unwrap();
            self.f_template.save("f_template.jpg").unwrap();
            let (rel_x, rel_y) = run_match_template(&f_area_cap_gray, &self.f_template, 0.2);
            
            if rel_x < 0 {
                continue;
            } 

            // otsu阈值分割获取f对应的文字
            let f_text_cap = self.capture_f_text(rel_y).unwrap();
            let f_text_cap_gray = grayscale(&f_text_cap);
            f_text_cap.save(format!("dumps2/{}_raw.jpg", cnt)).unwrap();
            cnt += 1;
            let otsu_thr = imageproc::contrast::otsu_level(&f_text_cap_gray);
            let f_text_cap_bin: ImageBuffer<Luma<u8>, Vec<u8>> = imageproc::contrast::threshold(&f_text_cap_gray, otsu_thr);
            
            // f_text_cap_bin.save("f_text_bin.jpg").unwrap();
            
            let bin_resized = imageops::resize(&f_text_cap_bin, 145, 32, imageops::FilterType::Gaussian);

            let mut padded_image = ImageBuffer::new(384, 32);
            padded_image.copy_from(&bin_resized, 0, 0).unwrap();

            let vec: Vec<f32> = padded_image.pixels().map(|p| p[0] as f32 / 255.0).collect();
            let raw_img = RawImage {
                data: vec,
                h: padded_image.height(),
                w: padded_image.width(),
            };
            // raw_img.to_gray_image().save("cao.jpg");
            // processed_img.to_gray_image().save("processed.jpg");

            // 还需要缩放到 32, x
            println!("h: {}, w: {}", raw_img.h, raw_img.w);
            
            let inference_result = self.model.inference_string(&raw_img);
            
            if ! self.all_list.contains(&inference_result) {
                warn!("not in all list: {}", inference_result);
                continue;
            }
            if self.black_list.contains(&inference_result) {
                warn!("black list: {}", inference_result);
                continue;
            }
            info!("pick up: {}", inference_result);

            self.enigo.key_down(enigo::Key::Layout('f'));
            sleep(12);
            self.enigo.key_up(enigo::Key::Layout('f'));

            self.enigo.mouse_scroll_y(-1);
            // common::error_and_quit("test");

        }
    }
}
