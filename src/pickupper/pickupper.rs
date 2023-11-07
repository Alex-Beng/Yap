use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::time::SystemTime;
use std::sync::{Arc, Mutex, RwLock};


use crate::common::sleep;
use crate::inference;
use crate::inference::img_process::{run_match_template, rgb_to_l, ContourFeatures};
use crate::{info::PickupInfo, common};
use crate::inference::inference::CRNNModel;
use crate::capture::{RawCaptureImage, self, PixelRect, RawImage};

use image::imageops::{grayscale, self, crop};
use image::{GrayImage, ImageBuffer, Luma, ColorType, GenericImage, DynamicImage};
use imageproc::contours;
use imageproc::contrast::adaptive_threshold;
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
    f_contour_feat: ContourFeatures,
    
    // 黑名单
    black_list: HashSet<String>,
    // 所有物品
    all_list: HashSet<String>,
    // 白名单
    white_list: HashSet<String>,

    // 配置
    config: PickupCofig,

    // for pickup error filte
    last_pickup_loop_cnt: i32,
    last_pickup_name: String,
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

        let mut f_contour_feat = ContourFeatures::new_empty();
        // 根据template计算contour特征
        let _img = template.clone();
        let _img_bin = adaptive_threshold(&_img, 14); // block 大小是否需要自适应？
        let _contours:Vec<imageproc::contours::Contour<u32>> = imageproc::contours::find_contours(&_img_bin);
        for i in 0.._contours.len() {
        // for _contour in _contours {
            let _contour = &_contours[i];
            let _contour = imageproc::contours::Contour {
                points: _contour.points.clone(),
                border_type: _contour.border_type,
                parent: _contour.parent,
            };

            // 这个FFF有爸爸轮廓的只有那个F
            if _contour.parent.is_some() {
                let _contour_clone = imageproc::contours::Contour {
                    points: _contour.points.clone(),
                    border_type: _contour.border_type,
                    parent: _contour.parent,
                };
                let _contour_father = &_contours[_contour.parent.unwrap()];
                let _contour_father = imageproc::contours::Contour {
                    points: _contour_father.points.clone(),
                    border_type: _contour_father.border_type,
                    parent: _contour_father.parent,
                };

                f_contour_feat = ContourFeatures::new(_contour, _contour_father, true, &template);
                
                // hard code here for feat
                // 受不了这个傻逼FFF.bmp了，直接硬编码得了
                f_contour_feat.bbox_wh_ratio = 0.7;
                f_contour_feat.area_ratio = 0.010997644;
                f_contour_feat.bbox_area_avg_pixel = 178.59644;
                f_contour_feat.contour_points_avg_pixel = 241.72464;
                
                // println!("f_contour_feat: {:?}", f_contour_feat);
            }
        }
        
        Pickupper {
            model: CRNNModel::new(String::from("model_training.onnx"), String::from("index_2_word.json")),
            enigo: Enigo::new(),

            f_template: template,
            f_contour_feat: f_contour_feat,

            black_list: bk_list,
            all_list: all_list,
            white_list: wt_list,

            config: config,

            last_pickup_loop_cnt: -1,
            last_pickup_name: String::new(),
        }
    }
    

    pub fn start(&mut self) -> ! {
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
        let mut loop_cnt = -1;
        loop {
            loop_cnt += 1;
            { 
                let infer_gap = self.config.infer_gap.read().unwrap();
                sleep(*infer_gap);
            }
            // 输出一次loop时间
            // info!("loop time: {}ms", start_time.elapsed().unwrap().as_millis());
            start_time = SystemTime::now();

            
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
            let mut f_area_cap_le = f_area_cap.clone();
            // warn!("f_area_cap: w: {}, h: {}", f_area_cap.width(), f_area_cap.height());
            // info!("f_template: w: {}, h: {}", self.f_template.width(), self.f_template.height());
            
            let f_area_cap_gray: GrayImage;
            if use_l {
                f_area_cap_gray = rgb_to_l(&f_area_cap.to_rgb8());
            }
            else {
                f_area_cap_gray = grayscale(&f_area_cap);
            }

            let temp_match_time = SystemTime::now();
            // f_area_cap_gray.save("farea.jpg").unwrap();
            // self.f_template.save("f_template.jpg").unwrap();
            // let (rel_x1, rel_y1) = run_match_template(&f_area_cap_gray, &self.f_template, temp_thre);
            // println!("rel_y1: {}", rel_y1);

            // contour matching 
            let f_area_cap_thre = adaptive_threshold(&f_area_cap_gray, 14);
            let f_area_contours: Vec<contours::Contour<u32>> = imageproc::contours::find_contours(&f_area_cap_thre);
            
            let mut f_cnt = 0;
            let mut rel_x = -1;
            let mut rel_y = -1;

            let mut best_match = 0.;
            let mut best_father_bbox = PixelRect {
                left: 0,
                top: 0,
                width: 0,
                height: 0,
            };

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
                    &f_area_cap_gray
                );
                
                let (cos_simi, _valid) = self.f_contour_feat.can_match(&contour_feat, 0.995);
                if contour_feat.contour_have_father == true && _valid {
                    f_cnt += 1;

                    // compute the rel x and y
                    
                    // 使用父亲轮廓bbox的计算
                    let father_contour = &f_area_contours[contour.parent.unwrap()];
                    let father_contour = imageproc::contours::Contour {
                        points: father_contour.points.clone(),
                        border_type: father_contour.border_type,
                        parent: father_contour.parent,
                    };
                    let father_contour_bbox = inference::img_process::contours_bbox(father_contour);

                    rel_y = father_contour_bbox.top;
                    rel_x = father_contour_bbox.left;
                    // rel_y = bbox.top - self.config.info.f_area_position.top;
                    // rel_x = 1;
                    if cos_simi > best_match {
                        best_match = cos_simi;
                        best_father_bbox = father_contour_bbox;
                    }

                }
                
            }
            // println!(", rel_y2: {}; {}, {}, {}", rel_y, f_cnt, self.config.info.pickup_y_gap, rel_y-rel_y1);

            // warn!("temp match time: {}ms", temp_match_time.elapsed().unwrap().as_millis());
            // info!("best_match: {}, f_cnt: {}", best_match, f_cnt);
            if rel_x < 0 || f_cnt != 1 || best_match < 0.995 {
                // // 说明没有找到，保存全图
                // if full_cnt % 20 == 0 {
                //     game_window_cap.save(format!("{}/{}_full.jpg", "./dumps_full", full_cnt)).unwrap();
                // }
                // full_cnt += 1;
                continue;
            }
            

            if true {
                info!("best_match: {}", best_match);
                // crop the gray image using best_father_bbox
                let F_father_cap = crop(&mut f_area_cap_le,
                    best_father_bbox.left as u32,
                    best_father_bbox.top as u32,
                    best_father_bbox.width as u32,
                    best_father_bbox.height as u32,
                );
                // F_father_cap.to_image().save(format!("{}_F_father.jpg", loop_cnt)).unwrap();
                // f_area_cap_le.save(format!("{}_f_area_cap_le.jpg", loop_cnt)).unwrap();
            }
            
            let infer_time = SystemTime::now();

            let mut res_strings: Vec<String> = vec![String::new(); 5];
            
            // check yi == 2 first
            let mut yi = 2;
            let mut pk_infer_end = false;
            while ! pk_infer_end {
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
                
                // let otsu_thr = imageproc::contrast::otsu_level(&f_text_cap_gray);
                // let f_text_cap_bin: ImageBuffer<Luma<u8>, Vec<u8>> = imageproc::contrast::threshold(&f_text_cap_gray, otsu_thr);

                // transfer to adaptive threshold
                // let f_text_cap_bin = adaptive_threshold(&f_text_cap_gray, 14);

                let f_text_cap_bin = f_text_cap_gray;

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

                // 更新yi的状态机
                match yi {
                    2 => {
                        yi = 0;
                        if res_strings[2] == "" {
                            pk_infer_end = true;
                            warn!("中间为空，不拾取");
                        }
                    }
                    0 => {
                        yi = 1;
                    }
                    1 => {
                        yi = 3;
                    }
                    3 => {
                        yi = 4;
                    }
                    4 => {
                        break;
                    }
                    _ => {
                        warn!("yi error: {}", yi);
                        pk_infer_end = true;
                    }
                }
            }
            if pk_infer_end {
                continue;
            }

            // info!("infer time: {}ms", infer_time.elapsed().unwrap().as_millis());

            // 输出所有推理结果
            info!("推理结果: ");
            for i in 0..5 {
                info!("{}: {}", i, res_strings[i as usize]);
            }
            self.do_pickups(res_strings, loop_cnt);
        
        }
    }
    pub fn do_pickups(&mut self, infer_res: Vec<String>, loop_cnt: i32) {
        let mut infer_res = infer_res;
        let do_pk = self.config.do_pickup.lock().unwrap();
        let f_inter = self.config.f_inter.read().unwrap();
        let f_gap = self.config.f_gap.read().unwrap();
        let scroll_gap = self.config.scroll_gap.read().unwrap();

        // 规划的最终动作
        // 0: do F
        // -1: scroll down -1
        // 1: scroll up 1
        // if infer_res[2] == "" {
        //     info!("中间为空，不拾取");
        //     return;
        // }
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
        // 检查是否仅有调查，其余为空
        let mut only_investigate = infer_res[2] == "调查";
        for i in 0..5 {
            if infer_res[i as usize] != ""  && infer_res[i as usize] != "调查" {
                only_investigate = false;
                break;
            }
        }
        if only_investigate {
            warn!("仅有调查，彻底疯狂！， F for 10 times");
            for _ in 0..10 {
                // copy from logi macro
                self.enigo.mouse_scroll_y(-1);
                sleep(10);
                self.enigo.key_down(enigo::Key::Layout('f'));
                sleep(10);
                self.enigo.key_up(enigo::Key::Layout('f'));
                sleep(10);
                self.enigo.mouse_scroll_y(-1);
                self.enigo.key_down(enigo::Key::Layout('f'));
                sleep(10);
                self.enigo.key_up(enigo::Key::Layout('f'));
            }
            return;
        }

        let up2sum = is_pks[0] + is_pks[1];
        let dn2sum = is_pks[3] + is_pks[4];
        let mut need_pks_sum: i32 = need_pks.iter().sum();
        
        if need_pks_sum == 0 {
            if up2sum <= dn2sum && dn2sum > 0 {
                // ops.push(-1);
                // ops.push(-1);
                for _ in 0..dn2sum {
                    ops.push(-1);
                }
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
                info!("{}, {}, {}, {}", self.last_pickup_name, infer_res[t_curr_f], self.last_pickup_loop_cnt, loop_cnt);
                if self.last_pickup_name == infer_res[t_curr_f] && (loop_cnt - self.last_pickup_loop_cnt < 4 &&
                loop_cnt != self.last_pickup_loop_cnt) {
                    warn!("连续两次拾取相同物品，不拾取");
                    // self.last_pickup_loop_cnt = loop_cnt; // in case more than twice
                    return;
                }

                ops.push(0);
                self.last_pickup_name = infer_res[t_curr_f].clone();
                self.last_pickup_loop_cnt = loop_cnt;
            

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

