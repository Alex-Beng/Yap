pub mod window_info;

use crate::capture::PixelRectBound;
use crate::info::window_info::{WINDOW_16_9, WINDOW_16_10};



#[derive(Clone, Debug)]
pub struct PickupInfo {
    pub left: i32,
    pub top: i32,
    
    pub width: u32,
    pub height: u32,

    pub f_area_position: PixelRectBound,
    pub f_template_w: u32,
    pub f_template_h: u32,
    pub f_alpha_left: u32,

    pub pickup_x_beg: u32,
    pub pickup_x_end: u32,

    pub pickup_y_gap: u32,

    pub artifact_put_in_x: u32,
    pub artifact_put_in_y: u32,
    pub artifact_upgrade_x: u32,
    pub artifact_upgrade_y: u32,
    pub artifact_skip_x: u32,
    pub artifact_skip1_y: u32,
    pub artifact_skip2_y: u32,

    pub online_challange_position: PixelRectBound,
    pub online_challange_confirm_x: u32,
    pub online_challange_confirm_y: u32,

    pub uid_pos: PixelRectBound,

    pub tp_botton_pos: PixelRectBound,
}

impl PickupInfo {
    pub fn from_16_9(width: u32, height: u32, left: i32, top: i32) -> PickupInfo {
        WINDOW_16_9.to_pickup_info(height as f64, width as f64, left, top)
    }
    pub fn from_16_10(width: u32, height: u32, left: i32, top: i32) -> PickupInfo {
        WINDOW_16_10.to_pickup_info(height as f64, width as f64, left, top)
    }
}