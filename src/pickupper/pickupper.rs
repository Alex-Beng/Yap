use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::time::SystemTime;
use std::sync::{Arc, Mutex, RwLock};


use crate::common::sleep;
use crate::inference;
use crate::inference::img_process::{run_match_template, rgb_to_l};
use crate::{info::PickupInfo, common};
use crate::inference::inference::CRNNModel;
use crate::capture::{RawCaptureImage, self, PixelRect, RawImage};

use image::imageops::{grayscale, self, crop};
use image::{GrayImage, ImageBuffer, Luma, ColorType, GenericImage, DynamicImage};
use imageproc::definitions::Image;
use tract_onnx::prelude::*;
use serde_json;
use enigo::*;
use log::{info, warn, trace};


pub struct PickupCofig {
    pub info: PickupInfo,
    pub bw_path: String,
    pub use_l: bool,
    pub dump: bool,
    pub dump_path: String,
    pub dump_cnt: u32,
    pub temp_thre: f32,
    pub do_pickup: Arc<Mutex<bool>>,
    pub infer_gap: Arc<RwLock<u32>>,
    pub f_inter: Arc<RwLock<u32>>,
    pub f_gap: Arc<RwLock<u32>>,
    pub scroll_gap: Arc<RwLock<u32>>,
}

pub struct Pickupper {
    model: CRNNModel,
    enigo: Enigo,

    f_template: GrayImage,
    
    // 黑名单
    black_list: HashSet<String>,
    // 所有物品
    all_list: HashSet<String>,
    // 白名单
    white_list: HashSet<String>,

    // 配置
    config: PickupCofig,
}

impl Pickupper {
    pub fn new(config: PickupCofig) -> Pickupper {
        let info = &config.info;
        let mut bk_list: HashSet<String> = HashSet::new();
        let mut wt_list: HashSet<String> = HashSet::new();

        
        // let black_list_path = "./black_lists.json";
        let black_list_path = format!("{}/{}", config.bw_path, "black_lists.json");

        let mut file = File::open(black_list_path).expect("Failed to open black list file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read black list file");

        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let bk_items = json.as_array().unwrap();
        for item in bk_items {
            bk_list.insert(item.as_str().unwrap().to_string());
            info!("添加到黑名单: {}", item.as_str().unwrap().to_string());
            
        }
            
        // let white_list_path = "./white_lists.json";
        let white_list_path = format!("{}/{}", config.bw_path, "white_lists.json");
        let mut file = File::open(white_list_path).expect("Failed to open white list file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read white list file");

        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let white_items = json.as_array().unwrap();
        for item in white_items {
            wt_list.insert(item.as_str().unwrap().to_string());
            info!("添加到白名单: {}", item.as_str().unwrap().to_string());
            
        }

        let mut all_list: HashSet<String> = HashSet::new();
        let content = String::from(include_str!("../../models/all_list.json"));
        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let al_items = json.as_array().unwrap();
        for item in al_items {
            all_list.insert(item.as_str().unwrap().to_string());
        }

        let template_raw = image::load_from_memory(include_bytes!("../../models/FFF.bmp")).unwrap();
        let template: GrayImage;
        if config.use_l {
            template = rgb_to_l(&template_raw.to_rgb8());
        }
        else {
            template = grayscale(&template_raw);
        }
        // 需要对template进行缩放
        let template = imageops::resize(&template, info.f_template_w, info.f_template_h, imageops::FilterType::Gaussian);

        
        Pickupper {
            model: CRNNModel::new(String::from("model_training.onnx"), String::from("index_2_word.json")),
            enigo: Enigo::new(),

            f_template: template,

            black_list: bk_list,
            all_list: all_list,
            white_list: wt_list,

            config: config
        }
    }
    

    pub fn start(&mut self) {
        let dump = self.config.dump;
        let dump_path = self.config.dump_path.clone();
        let cnt = self.config.dump_cnt;
        
        let temp_thre = self.config.temp_thre;
        // let do_pickup = self.config.do_pickup.clone();
        let use_l = self.config.use_l;

        let mut cnt = cnt;

        let game_win_rect = PixelRect {
            left: self.config.info.left,
            top: self.config.info.top,
            width: self.config.info.width as i32,
            height: self.config.info.height as i32,
        };

        let mut start_time = SystemTime::now();
        // let mut full_cnt = 0;
        // let infer_gap_lock = self.config.infer_gap;
        loop {
            { 
                let infer_gap = self.config.infer_gap.read().unwrap();
                sleep(*infer_gap);
            }
            // 输出一次loop时间
            // info!("loop time: {}ms", start_time.elapsed().unwrap().as_millis());
            start_time = SystemTime::now();

            let temp_match_time = SystemTime::now();
            // 截一张全屏
            let mut game_window_cap = capture::capture_absolute_image(&game_win_rect).unwrap();
            // game_window_cap.save("game_window.jpg").unwrap();

            // 改为从window_cap中crop
            let f_area_cap = crop(&mut game_window_cap, 
                self.config.info.f_area_position.left as u32,
                self.config.info.f_area_position.top as u32,
                self.config.info.f_area_position.right as u32 - self.config.info.f_area_position.left as u32,
                self.config.info.f_area_position.bottom as u32 - self.config.info.f_area_position.top as u32);
            let f_area_cap = DynamicImage::ImageRgb8(f_area_cap.to_image());
            // warn!("f_area_cap: w: {}, h: {}", f_area_cap.width(), f_area_cap.height());
            // info!("f_template: w: {}, h: {}", self.f_template.width(), self.f_template.height());
            
            let f_area_cap_gray: GrayImage;
            if use_l {
                f_area_cap_gray = rgb_to_l(&f_area_cap.to_rgb8());
            }
            else {
                f_area_cap_gray = grayscale(&f_area_cap);
            }
            // f_area_cap_gray.save("farea.jpg").unwrap();
            // self.f_template.save("f_template.jpg").unwrap();
            let (rel_x, rel_y) = run_match_template(&f_area_cap_gray, &self.f_template, temp_thre);

            // info!("temp match time: {}ms", temp_match_time.elapsed().unwrap().as_millis());
            if rel_x < 0 {
                // // 说明没有找到，保存全图
                // if full_cnt % 20 == 0 {
                //     game_window_cap.save(format!("{}/{}_full.jpg", "./dumps_full", full_cnt)).unwrap();
                // }
                // full_cnt += 1;
                continue;
            }
            
            let infer_time = SystemTime::now();

            let mut res_strings: Vec<String> = vec![String::new(); 5];
            
            for yi in 0..5 {
                let y_offset = (yi - 2) * self.config.info.pickup_y_gap as i32 + rel_y as i32;

                let f_text_cap = crop(&mut game_window_cap,
                    self.config.info.pickup_x_beg as u32,
                        (self.config.info.f_area_position.top as i32 + y_offset) as u32,
                        self.config.info.pickup_x_end as u32 - self.config.info.pickup_x_beg as u32,
                        self.f_template.height() as u32,
                    );
                let f_text_cap = DynamicImage::ImageRgb8(f_text_cap.to_image());
                let f_text_cap_gray: GrayImage;
                if use_l {
                    f_text_cap_gray = rgb_to_l(&f_text_cap.to_rgb8());
                }
                else {
                    f_text_cap_gray = grayscale(&f_text_cap);
                }
                
                let otsu_thr = imageproc::contrast::otsu_level(&f_text_cap_gray);
                let f_text_cap_bin: ImageBuffer<Luma<u8>, Vec<u8>> = imageproc::contrast::threshold(&f_text_cap_gray, otsu_thr);
                // f_text_cap.save(format!("f_text_{}.jpg", yi)).unwrap();
                
                // 小绷，这个缩放至145x32是按最老的dump硬编码
                let bin_resized = imageops::resize(&f_text_cap_bin, 221, 32, imageops::FilterType::Gaussian);

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
                // println!("h: {}, w: {}", raw_img.h, raw_img.w);
                

                // let t1 = SystemTime::now();
                let inference_result = self.model.inference_string(&raw_img);
                // info!("inference 1 time: {}ms", t1.elapsed().unwrap().as_millis());

                if dump && inference_result != "" {
                    f_text_cap.save(format!("{}/{}_{}_{}_raw.jpg", dump_path, cnt, yi, inference_result)).unwrap();
                    f_text_cap_bin.save(format!("{}/{}_{}_{}_bin.jpg", dump_path, cnt, yi, inference_result)).unwrap();
                    cnt += 1;
                }

                res_strings[yi as usize] = inference_result;
            }

            // info!("infer time: {}ms", infer_time.elapsed().unwrap().as_millis());

            // 输出所有推理结果
            info!("推理结果: ");
            for i in 0..5 {
                info!("{}: {}", i, res_strings[i as usize]);
            }
            self.do_pickups(res_strings);
        
        }
    }
    pub fn do_pickups(&mut self, infer_res: Vec<String>) {
        let mut infer_res = infer_res;
        let do_pk = self.config.do_pickup.lock().unwrap();
        let f_inter = self.config.f_inter.read().unwrap();
        let f_gap = self.config.f_gap.read().unwrap();
        let scroll_gap = self.config.scroll_gap.read().unwrap();

        // 规划的最终动作
        // 0: do F
        // -1: scroll down -1
        // 1: scroll up 1
        if infer_res[2] == "" {
            info!("中间为空，不拾取");
            return;
        }
        let mut ops: Vec<i32> = Vec::new();

        let mut is_pks = vec![0, 0, 0, 0, 0];
        let mut need_pks = vec![0, 0, 0, 0, 0];
        for i in 0..5 {
            let s = &infer_res[i as usize];
            if s != "" {
                is_pks[i as usize] = 1;
            }
            if self.white_list.contains(s) || self.all_list.contains(s) && !self.black_list.contains(s) {
                need_pks[i as usize] = 1;
            }
        }
        let up2sum = is_pks[0] + is_pks[1];
        let dn2sum = is_pks[3] + is_pks[4];
        let mut need_pks_sum: i32 = need_pks.iter().sum();
        
        if need_pks_sum == 0 {
            if up2sum <= dn2sum && dn2sum > 0 {
                ops.push(-1);
                ops.push(-1);
            }
            else {
                // 11100 01100 
                if up2sum == 2 && dn2sum == 0 || up2sum == 1 && dn2sum == 0 {
                }
                // not 00100
                else if up2sum != 0 {
                    ops.push(1);
                }
            }
        }

        let mut t_curr_f = 2;
        while need_pks_sum > 0 {
            if need_pks[t_curr_f] == 1 {
                ops.push(0);
                need_pks_sum -= 1;
                if t_curr_f != 0 && t_curr_f == need_pks.len()-1 
                || t_curr_f != 0 && t_curr_f+1 < infer_res.len() && infer_res[t_curr_f+1] == "" {
                    t_curr_f -= 1;
                }
                need_pks.remove(t_curr_f);
                infer_res.remove(t_curr_f);

            }
            else {
                for i in 0..need_pks.len() {
                    if need_pks[i] == 1 {
                        if i < t_curr_f {
                            if t_curr_f - i == 1 {
                                ops.push(1);
                                t_curr_f -= 1;
                            }
                            else {
                                ops.push(1);
                                ops.push(1);
                                t_curr_f -= 2;
                            }
                        }
                        else {
                            if i - t_curr_f == 1 {
                                ops.push(-1);
                                t_curr_f += 1;
                            }
                            else {
                                ops.push(-1);
                                ops.push(-1);
                                t_curr_f += 2;
                            }
                        }
                        break;
                    }
                }
            }
        }

        info!("拾起动作: {:?}", ops);
        if !*do_pk {
            return;
        }

        for op in ops {
            if op == 0 {
                self.enigo.key_down(enigo::Key::Layout('f'));
                // sleep(50);
                sleep(*f_inter);
                self.enigo.key_up(enigo::Key::Layout('f'));
                // sleep(90);
                sleep(*f_gap);
            }
            else {
                self.enigo.mouse_scroll_y(op);
                // sleep(40);
                sleep(*scroll_gap);
            }
        }

    }
    
}

