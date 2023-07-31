pub mod window_info;

use crate::capture::PixelRectBound;
use crate::info::window_info::WINDOW_16_9;



#[derive(Clone, Debug)]
pub struct PickupInfo {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,

    pub f_area_position: PixelRectBound,
    pub f_template_w: u32,
    pub f_template_h: u32,

    pub pickup_x_beg: u32,
    pub pickup_x_end: u32,

    pub pickup_y_gap: u32,
}

impl PickupInfo {
    pub fn from_16_9(width: u32, height: u32, left: i32, top: i32) -> PickupInfo {
        WINDOW_16_9.to_pickup_info(height as f64, width as f64, left, top)
    }
}