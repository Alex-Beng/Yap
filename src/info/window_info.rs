use crate::info::PickupInfo;
use crate::capture::PixelRectBound;

pub struct Rect(f64, f64, f64, f64); // top, right, bottom, left

pub struct WindowInfo {
    pub width: f64,
    pub height: f64,

    pub f_area_pos: Rect,
    
    pub pickup_x_beg: f64,
    pub pickup_x_end: f64,
}

impl WindowInfo {
    pub fn to_pickup_info(&self, h: f64, w: f64, left: i32, top: i32) -> PickupInfo {
        let convert_rect = |rect: &Rect| {
            let top = rect.0 / self.height * h;
            let right = rect.1 / self.width * w;
            let bottom = rect.2 / self.height * h;
            let left = rect.3 / self.width * w;

            PixelRectBound {
                left: left as i32,
                top: top as i32,
                right: right as i32,
                bottom: bottom as i32,
            }
        };
        let convert_x = |x: f64| {
            x / self.width * w
        };
        
        // yap中无用233
        let convert_y = |y: f64| {
            y / self.height * h
        };
        PickupInfo { 
            f_area_position: convert_rect(&self.f_area_pos), 
            pickup_x_beg: convert_x(self.pickup_x_beg) as u32,
            pickup_x_end: convert_x(self.pickup_x_end) as u32,
            width: w as u32,
            height: h as u32,
            left,
            top,
        }
    }
}

pub const WINDOW_16_9: WindowInfo = WindowInfo {
    width: 1920.0,
    height: 1080.0,
    f_area_pos: Rect(340.0, 1157.0, 720.0, 1090.0),
    pickup_x_beg: 1218.0,
    pickup_x_end: 1426.0,
};