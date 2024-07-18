use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::SystemTime;
use std::sync::{Arc, Mutex, RwLock};


use crate::common::sleep;
use crate::inference::img_process::run_naive_alpha_triangle_matching;
use crate::inference::img_process::{rgb_to_l, ContourFeatures};
use crate::{info::PickupInfo, common};
use crate::inference::net_infer::CRNNModel;
use crate::capture::{self, PixelRect, RawImage};

use image::imageops::{grayscale, self, crop};
use image::{GrayImage, ImageBuffer, Luma, GenericImage, DynamicImage};
use imageproc::contours;
use imageproc::contrast::adaptive_threshold;
use serde_json;
use enigo::*;
use log::{info, warn};
use enigo::Coordinate;
use enigo::Direction;


pub struct PickupCofig {
    pub info: PickupInfo,
    pub hwnd: common::WindowHandle,
    pub bw_path: String,
    pub use_l: bool,
    pub press_y: bool,
    pub dump: bool,
    pub dump_path: String,
    pub dump_cnt: u32,
    pub pick_key: char,
    pub cosin_thr: f32,
    pub do_pickup: Arc<Mutex<bool>>,
    pub infer_gap: Arc<RwLock<u32>>,
    pub f_inter: Arc<RwLock<u32>>,
    pub f_gap: Arc<RwLock<u32>>,
    pub scroll_gap: Arc<RwLock<u32>>,
    pub click_tp: Arc<Mutex<bool>>,
    pub single_mode: Arc<Mutex<bool>>,
}

#[allow(dead_code)]
pub struct Pickupper {
    model: CRNNModel,
    enigo: Enigo,

    f_contour_feat: Vec<f32>,
    tp_botton_feat: Vec<f32>,
    
    // 合并黑白名单到hashmap: name -> pickup or not
    word2pick: HashMap<String, bool>,

    // 配置
    config: PickupCofig,

    // for pickup error filte
    last_pickup_loop_cnt: i32,
    last_pickup_name: String,
}

impl Pickupper {
    pub fn new(config: PickupCofig) -> Pickupper {
        let mut bk_list: HashSet<String> = HashSet::new();
        let mut wt_list: HashSet<String> = HashSet::new();

        // 编译期加载all list
        let mut all_list: HashSet<String> = HashSet::new();
        let content = String::from(include_str!("../../models/all_list.json"));
        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let al_items = json.as_array().unwrap();
        for item in al_items {
            all_list.insert(item.as_str().unwrap().to_string());
        }

        // 编译期加载默认黑名单
        let content = String::from(include_str!("../../models/default_black_list.json"));
        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let al_items = json.as_array().unwrap();
        for item in al_items {
            bk_list.insert(item.as_str().unwrap().to_string());
        }
        

        // 运行时加载自定义black & white list
        let config_list_path = format!("{}/{}", config.bw_path, "config.json");
        // 处理不存在config的异常，不存在则创建
        if !std::path::Path::new(config_list_path.as_str()).exists() {
            
            let _ = File::create(config_list_path.as_str()).expect("Failed to create config.json");
            
            let infer_gap_for_json = *config.infer_gap.read().unwrap();
            let f_inter_for_json = *config.f_inter.read().unwrap();
            let f_gap_for_json = *config.f_gap.read().unwrap();
            let scroll_gap_for_json = *config.scroll_gap.read().unwrap();
            let click_tp_for_json = *config.click_tp.lock().unwrap();
            let mut json = serde_json::json!({
                "black_list": [],
                "white_list": [],
                "infer_gap": infer_gap_for_json,
                "f_internal": f_inter_for_json,
                "f_gap": f_gap_for_json,
                "scroll_gap": scroll_gap_for_json,
                "click_tp": click_tp_for_json,
                "pick_key": "f",
                "cos_thre": 0.9977,
                "single_mode": false,
                "uid_mask": true,
                "press_y": true,
            });

            // 如果存在之前的black list or white list
            // 读取，以兼容旧版本
            let black_list = format!("{}/{}", config.bw_path, "black_lists.json");
            if std::path::Path::new(black_list.as_str()).exists() {
                info!("存在旧版本black_lists.json，将其合并到config.json");
                let mut file = File::open(black_list.as_str()).expect("Failed to open black_lists.json");
                let mut content = String::new();
                file.read_to_string(&mut content).expect("Failed to read black_lists.json");
                let old_bk: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
                // 删除已经在默认黑名单的
                let mut new_bk: Vec<String> = Vec::new();
                for item in old_bk.as_array().unwrap() {
                    if !bk_list.contains(item.as_str().unwrap()) {
                        new_bk.push(item.as_str().unwrap().to_string());
                        info!("添加到自定义黑名单: {}", item.as_str().unwrap().to_string())
                    }
                }
                // make new_bk json
                json["black_list"] = serde_json::json!(new_bk);
            }
            let white_list = format!("{}/{}", config.bw_path, "white_lists.json");
            if std::path::Path::new(white_list.as_str()).exists() {
                let mut file = File::open(white_list.as_str()).expect("Failed to open white_list.json");
                let mut content = String::new();
                file.read_to_string(&mut content).expect("Failed to read white_list.json");
                let old_wt: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
                json["white_list"] = old_wt;
            }

            let json_str = serde_json::to_string_pretty(&json).unwrap();
            std::fs::write(config_list_path, json_str).expect("Unable to write config.json");
        }

        let config_list_path = format!("{}/{}", config.bw_path, "config.json");
        let mut file = File::open(config_list_path).expect("Failed to config.json");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read config.json");

        let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let bk_items = json.as_object().unwrap().get("black_list").unwrap().as_array().unwrap();
        let wt_items = json.as_object().unwrap().get("white_list").unwrap().as_array().unwrap();

        for item in bk_items {
            bk_list.insert(item.as_str().unwrap().to_string());
            info!("添加到黑名单: {}", item.as_str().unwrap().to_string());
        }

        for item in wt_items {
            wt_list.insert(item.as_str().unwrap().to_string());
            info!("添加到白名单: {}", item.as_str().unwrap().to_string());
        }
        let mut word2pick: HashMap<String, bool> = HashMap::new();
        // 设全部为true
        for item in all_list {
            word2pick.insert(item, true);
        }
        // 拉黑bk_item
        for item in bk_list {
            word2pick.insert(item, false);
        }
        // 拉白wt_item
        for item in wt_list {
            word2pick.insert(item, true);
        }


        let mut f_contour_feat = ContourFeatures::new_empty();
        // hard code here for feat
        f_contour_feat.bbox_wh_ratio = 0.7;
        f_contour_feat.area_ratio = 0.010997644;
        f_contour_feat.bbox_area_avg_pixel = 178.59644;
        f_contour_feat.contour_points_avg_pixel = 241.72464;
        f_contour_feat.contour_len2_area_ratio =  0.8632692;
        f_contour_feat.father_bbox_wh_ratio = 1.21875;
        let f_contour_feat = f_contour_feat.to_features_vec();

        // 计算tp button的feat_vec
        let tp_button_feat = vec![1., 1., 1., 1., 1.]; // tp_button_feat.to_features_vec();

        
        Pickupper {
            model: CRNNModel::new(String::from("model_training.onnx"), String::from("index_2_word.json")),
            enigo: Enigo::new(&Settings::default()).unwrap(),

            f_contour_feat,
            tp_botton_feat: tp_button_feat,

            word2pick,

            config,

            last_pickup_loop_cnt: -1,
            last_pickup_name: String::new(),
        }
    }
    
    #[allow(unused_variables, unused_assignments)]
    pub fn start(&mut self) -> ! {
        // 用于秘境挑战邀请的自动点击
        let (tx, rx) = std::sync::mpsc::channel::<DynamicImage>();
        // 用于自动点击传送按钮
        let (tx_tp, rx_tp) = std::sync::mpsc::channel::<DynamicImage>();

        let model_online = CRNNModel::new(String::from("model_training.onnx"), String::from("index_2_word.json"));
        let mut enigo_online = Enigo::new(&Settings::default()).unwrap();
        let online_confirm_x = self.config.info.online_challange_confirm_x + self.config.info.left as u32;
        let online_confirm_y = self.config.info.online_challange_confirm_y + self.config.info.top as u32;
        if self.config.press_y {
            thread::spawn(move || {
                let mut cnt = 0;
                loop {
                    cnt += 1;

                    let img = rx.recv().unwrap();
                    
                    let img_gray = grayscale(&img);
                    
                    let bin_resized = imageops::resize(&img_gray, 199, 32, imageops::FilterType::Gaussian);
                    let mut padded_image = ImageBuffer::new(384, 32);
                    padded_image.copy_from(&bin_resized, 0, 0).unwrap();
                    let vec: Vec<f32> = padded_image.pixels().map(|p| p[0] as f32 / 255.0).collect();
                    let raw_img = RawImage {
                        data: vec,
                        h: padded_image.height(),
                        w: padded_image.width(),
                    };
                    let inference_result = model_online.inference_string(&raw_img);
                    
                    // if inference_result != "" {
                    //     img.save(format!("dumps4.2/77/{}_0_秘境组队邀请_raw.jpg", cnt)).unwrap();
                    // }

                    if inference_result.is_empty() {
                        continue;
                    }
                    info!("online challage inference_result: {}", inference_result);
                    if inference_result == "秘境挑战组队邀请" || inference_result == "进入世界申请（" {
                        // press Y first
                        enigo_online.key(Key::Y, Direction::Press).unwrap();
                        sleep(50);
                        enigo_online.key(Key::Y, Direction::Release).unwrap();

                        // move mouse to the right position
                        // enigo_online.mouse_move_to(online_confirm_x as i32, online_confirm_y as i32);
                        // sleep(50);

                        // click
                        // enigo_online.mouse_click(enigo::MouseButton::Left);
                        // sleep(50);
                    }
                }
            });
        }

        let botton_feat = self.tp_botton_feat.clone();
        let mut enigo_tp = Enigo::new(&Settings::default()).unwrap();
        let botton_click_x = 
            (self.config.info.tp_botton_pos.left + self.config.info.tp_botton_pos.right) / 2 + self.config.info.left;
        let botton_click_y =
            (self.config.info.tp_botton_pos.top + self.config.info.tp_botton_pos.bottom) / 2 + self.config.info.top;
        let click_lock = self.config.click_tp.clone();
        thread::spawn(move || {
            
            loop {
                let do_click = *click_lock.lock().unwrap();

                let img = rx_tp.recv().unwrap();
                let img_gray = grayscale(&img);

                let roi_bin = adaptive_threshold(&img_gray, 41);
                let contours: Vec<contours::Contour<u32>> = imageproc::contours::find_contours(&roi_bin);
                let mut no_father_cnt = 0;
                let mut best_match = 0.;

                for contour in &contours {
                    if contour.parent.is_some() { continue; }
                    let contour_clone = imageproc::contours::Contour {
                        points: contour.points.clone(),
                        border_type: contour.border_type,
                        parent: contour.parent,
                    };

                    let contour_feat = ContourFeatures::new_tp(
                        contour_clone,
                        &img_gray
                    );
                    let (cos_simi, _valid) = contour_feat.can_match_tp(&botton_feat, 0.999);
                    if _valid {
                        // info!("{} {}, {}", contour_feat.area_ratio, contour_feat.bbox_wh_ratio, cos_simi);
                        // info!("{} {} {}", contour_feat.bbox_area_avg_pixel, contour_feat.contour_points_avg_pixel, contour_feat.contour_len2_area_ratio);
                        // info!("{:?}", contour_feat.to_feature_vec_tp());
                        no_father_cnt += 1;
                        if cos_simi > best_match {
                            best_match = cos_simi;
                        }
                    }

                } 
                // info!("{} {}, {} {} {}", botton_feat[0], botton_feat[1], botton_feat[2], botton_feat[3], botton_feat[4]);
                // info!("{:?}", botton_feat);
                
                // info!("no_father_cnt: {}", no_father_cnt);
                // info!("tp button click with simi: {}", best_match);
                if no_father_cnt != 1 {
                    continue;
                }
                if best_match > 0.7 && do_click {
                    info!("tp button click with simi: {}", best_match);
                    // move to click
                    enigo_tp.move_mouse(botton_click_x as i32, botton_click_y as i32, Coordinate::Abs).unwrap();
                    sleep(25);
                    
                    enigo_tp.button(Button::Left, Direction::Click).unwrap();
                    // sleep(500);
                }

            }
        });

        let dump = self.config.dump;
        let dump_path = self.config.dump_path.clone();
        let cnt = self.config.dump_cnt;
        
        // let do_pickup = self.config.do_pickup.clone();
        let use_l = self.config.use_l;

        let mut cnt = cnt;

        let game_win_rect = PixelRect {
            left: self.config.info.left,
            top: self.config.info.top,
            width: self.config.info.width as i32,
            height: self.config.info.height as i32,
        };
        let cos_thre = self.config.cosin_thr;

        // text 高度，这里先硬编码成 30 / 72 * info.pickup_y_gap
        let text_h = (30. / 72. * self.config.info.pickup_y_gap as f64) as u32;

        let mut start_time = SystemTime::now();
        let mut full_cnt = 0;
        // let infer_gap_lock = self.config.infer_gap;
        let mut loop_cnt = -1;
        let mut last_online_challage_time = SystemTime::now();
        let last_tp_click_time = SystemTime::now();
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
            let game_window_cap_rgba = DynamicImage::ImageRgba8(capture::capture_absolute_image(self.config.hwnd, &game_win_rect).unwrap());
            // split it to rgb + alpha 
            // convert to rgb
            let mut game_window_cap = game_window_cap_rgba.to_rgb8();
            // 获取 alpha 通道图
            let alpha = game_window_cap_rgba.to_rgba8().pixels().map(|p| p[3]).collect::<Vec<u8>>();
            let mut alpha: ImageBuffer<Luma<u8>, Vec<_>> = ImageBuffer::from_vec(game_window_cap.width(), game_window_cap.height(), alpha).unwrap();
            // alpha.save("le.jpg");
            
            
            // if loop_cnt == 10 {
            //     game_window_cap_rgba.save("game_window.png").unwrap();
            //     process::exit(0);
            // }

            // let mut game_window_cap 
            // game_window_cap.save("game_window.jpg").unwrap();

            // 改为从window_cap中crop
            let mut f_area_cap = None;
            let mut f_ares_cap_alpha = None;
            {
                f_area_cap = Some(crop(&mut game_window_cap, 
                    self.config.info.f_area_position.left as u32,
                    self.config.info.f_area_position.top as u32,
                    self.config.info.f_area_position.right as u32 - self.config.info.f_area_position.left as u32,
                    self.config.info.f_area_position.bottom as u32 - self.config.info.f_area_position.top as u32).to_image());
                f_ares_cap_alpha = Some(crop(&mut alpha, 
                    self.config.info.f_alpha_left,
                    self.config.info.f_area_position.top as u32,
                    self.config.info.f_area_position.right as u32 - self.config.info.f_alpha_left,
                    self.config.info.f_area_position.bottom as u32 - self.config.info.f_area_position.top as u32).to_image());
            }
            let f_area_cap = f_area_cap.unwrap();
            let f_ares_cap_alpha = f_ares_cap_alpha.unwrap();

            // f_ares_cap_alpha.save("le.jpg").unwrap();
            // alpha.save("le2.jpg").unwrap();
            
            // 再crop秘境挑战的 
            if self.config.press_y && last_online_challage_time + std::time::Duration::from_secs(1) < SystemTime::now() {
                last_online_challage_time = SystemTime::now();
                let online_challage_cap = crop(&mut game_window_cap, 
                    self.config.info.online_challange_position.left as u32,
                    self.config.info.online_challange_position.top as u32,
                    self.config.info.online_challange_position.right as u32 - self.config.info.online_challange_position.left as u32,
                    self.config.info.online_challange_position.bottom as u32 - self.config.info.online_challange_position.top as u32);
                // 塞进channel
                let online_challage_cap = DynamicImage::ImageRgb8(online_challage_cap.to_image());
                tx.send(online_challage_cap).unwrap();
            }

            // 再crop传送按钮的
            if last_tp_click_time + std::time::Duration::from_millis(100) < SystemTime::now() {
                let tp_botton_roi = crop(&mut game_window_cap, 
                    self.config.info.tp_botton_pos.left as u32,
                    self.config.info.tp_botton_pos.top as u32,
                    self.config.info.tp_botton_pos.right as u32 - self.config.info.tp_botton_pos.left as u32,
                    self.config.info.tp_botton_pos.bottom as u32 - self.config.info.tp_botton_pos.top as u32);
                let tp_botton_roi = DynamicImage::ImageRgb8(tp_botton_roi.to_image());
                tx_tp.send(tp_botton_roi).unwrap();
            }
            
            let f_area_cap = DynamicImage::ImageRgb8(f_area_cap);
            let f_ares_cap_alpha = GrayImage::from(f_ares_cap_alpha);
            
            let f_area_cap_gray: GrayImage = if use_l {
                rgb_to_l(&f_area_cap.to_rgb8())
            } else {
                grayscale(&f_area_cap)
            };

            let temp_match_time = SystemTime::now();
            // f_area_cap_gray.save("farea.jpg").unwrap();

            // contour matching 
            // let (rel_x, rel_y) = run_contours_cosine_matching(&f_area_cap_gray, &self.f_contour_feat, cos_thre);

            // aplha triangle matching
            // let (rel_x, rel_y) = run_alpha_triangle_matching(&f_ares_cap_alpha, text_h/2);
            
            // rollback to the most naive version
            let (rel_x, rel_y) = run_naive_alpha_triangle_matching(&f_ares_cap_alpha, text_h/2);
            // f_ares_cap_alpha.save("f_ares_cap_alpha.jpg").unwrap();

            // warn!("temp match time: {}ms", temp_match_time.elapsed().unwrap().as_millis());
            // info!("best_match: {}, f_cnt: {}", best_match, f_cnt);
            if false {
                if full_cnt % 20 == 0 {
                    info!("save full");
                    game_window_cap.save(format!("{}/{}.jpg", "./dumps4.2/full", full_cnt)).unwrap();
                }
                full_cnt += 1;
            }
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
            
            // check yi == 2 first
            let mut yi = 2;
            let mut pk_infer_end = false;
            while ! pk_infer_end {
                let y_offset = (yi - 2) * self.config.info.pickup_y_gap as i32 + rel_y;

                let f_text_cap = crop(&mut game_window_cap,
                    self.config.info.pickup_x_beg,
                        (self.config.info.f_area_position.top + y_offset) as u32,
                        self.config.info.pickup_x_end - self.config.info.pickup_x_beg,
                        text_h
                    );
                let f_text_cap = DynamicImage::ImageRgb8(f_text_cap.to_image());
                let f_text_cap_gray: GrayImage = if use_l {
                    rgb_to_l(&f_text_cap.to_rgb8())
                }
                else {
                    grayscale(&f_text_cap)
                };
                
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

                // dump 不认识的和需要捡的
                if dump && !inference_result.is_empty() && !self.word2pick.contains_key(&inference_result) || 
                dump && self.word2pick.contains_key(&inference_result) && self.word2pick[&inference_result] {
                    // remove ? / : * " < > | \ / in file name
                    let inference_result = inference_result.replace(['?', '/', ':', '*', '"', '<', '>', '|', '\\', '/'], "");
                    f_text_cap.save(format!("{}/{}_{}_{}_raw.jpg", dump_path, cnt, yi, inference_result)).unwrap();
                    // f_text_cap_bin.save(format!("{}/{}_{}_{}_bin.jpg", dump_path, cnt, yi, inference_result)).unwrap();
                    cnt += 1;
                }

                res_strings[yi as usize] = inference_result;

                {
                    let single_mode = *self.config.single_mode.lock().unwrap();
                    if single_mode {
                        // info!("单次模式，推理结果: {}", res_strings[2]);
                        if res_strings[2].is_empty() {
                            pk_infer_end = true;
                        }
                        break;
                    }
                }

                // 更新yi的状态机
                match yi {
                    2 => {
                        yi = 1;
                        if res_strings[2].is_empty() {
                            pk_infer_end = true;
                            warn!("中间为空，不拾取");
                        }
                    }
                    1 => {
                        if res_strings[1].is_empty() {
                            yi = 3;
                        }
                        else {
                            yi = 0;
                        }
                    }
                    0 => {
                        yi = 3;
                    }
                    3 => {
                        if res_strings[3].is_empty() {
                            break;
                        }
                        else {
                            yi = 4;
                        }
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
    #[allow(clippy::collapsible_else_if)]
    pub fn do_pickups(&mut self, infer_res: Vec<String>, loop_cnt: i32) {
        let mut infer_res = infer_res;
        let do_pk = *self.config.do_pickup.lock().unwrap();
        let f_inter = *self.config.f_inter.read().unwrap();
        let f_gap = *self.config.f_gap.read().unwrap();
        let scroll_gap = *self.config.scroll_gap.read().unwrap();
        let f_truly_key = Key::Unicode(self.config.pick_key.to_ascii_lowercase());

        // 规划的最终动作
        // 0: do F
        // -1: scroll down -1
        // 1: scroll up 1
        // if infer_res[2] == "" {
        //     info!("中间为空，不拾取");
        //     return;
        // }
        let mut ops: Vec<i32> = Vec::new();

        let mut is_pks = [0, 0, 0, 0, 0];
        let mut need_pks = vec![0, 0, 0, 0, 0];
        let mut need_pks_cnt = 0;
        let mut all_is_need = true;
        // 是否全是调查，进行一个彻底的疯狂
        let mut is_all_investigate = self.word2pick["调查"];
        for i in 0..5 {
            let s = &infer_res[i as usize];
            if !s.is_empty() {
                is_pks[i as usize] = 1;
                if s != "调查" {
                    is_all_investigate = false;
                }
            }
            else {
                continue;
            }
            if self.word2pick.contains_key(s) && self.word2pick[s] {
                need_pks[i as usize] = 1;
                need_pks_cnt += 1;
            }
            else {
                all_is_need = false;
            }
        }
        // 检查是否全是所需物品
        if do_pk && (all_is_need && need_pks_cnt > 1 || is_all_investigate) {
            let f_times = need_pks_cnt + 1;
            warn!("仅有所需/调查点，彻底疯狂！， F for {}(10) times", f_times);
            for _ in 0..10 {
                // copy from logi macro
                self.enigo.scroll(1, Axis::Vertical).unwrap();
                sleep(10);
                self.enigo.key(f_truly_key, Direction::Press).unwrap();
                sleep(10);
                self.enigo.key(f_truly_key, Direction::Release).unwrap();
                sleep(10);
                self.enigo.scroll(-1, Axis::Vertical).unwrap();
                self.enigo.key(f_truly_key, Direction::Press).unwrap();
                sleep(10);
                self.enigo.key(f_truly_key, Direction::Release).unwrap();
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
                if (up2sum == 2 || up2sum == 1) && dn2sum == 0 {
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
                self.last_pickup_name.clone_from(&infer_res[t_curr_f]);
                self.last_pickup_loop_cnt = loop_cnt;
            

                need_pks_sum -= 1;
                if t_curr_f != 0 && t_curr_f == need_pks.len()-1 
                || t_curr_f != 0 && t_curr_f+1 < infer_res.len() && infer_res[t_curr_f+1].is_empty() {
                    t_curr_f -= 1;
                }
                need_pks.remove(t_curr_f);
                infer_res.remove(t_curr_f);

            }
            else {
                for (i, nd_pk) in need_pks.iter().enumerate() {
                    if *nd_pk == 1 {
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
        if !do_pk {
            return;
        }

        for op in ops {
            if op == 0 {
                self.enigo.key(f_truly_key, Direction::Press).unwrap();
                // sleep(50);
                sleep(f_inter);
                self.enigo.key(f_truly_key, Direction::Release).unwrap();
                // sleep(90);
                sleep(f_gap);
            }
            else {
                // 草为什么不同电脑上翻和下翻不一样
                self.enigo.scroll(-op, Axis::Vertical).unwrap();
                // sleep(40);
                sleep(scroll_gap);
            }
        }

    }
    
}

