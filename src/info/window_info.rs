use crate::info::PickupInfo;
use crate::capture::PixelRectBound;

pub struct Rect(f64, f64, f64, f64); // top, right, bottom, left

pub struct WindowInfo {
    pub width: f64,
    pub height: f64,

    // F键区域
    pub f_area_pos: Rect,
    // 模板宽高
    pub f_template_w: f64,
    pub f_template_h: f64,
    // 使用alpha通道时，左边界的x坐标
    pub f_area_left_alpha: f64,
    
    // 拾取框的x坐标范围
    pub pickup_x_beg: f64,
    pub pickup_x_end: f64,

    // 拾取物y方向上的间隔
    pub pickup_y_gap: f64,

    // 快速强化圣遗物所需的点位
    // 固定动作，无需感知
    // pub in -> upgrade -> switch right to skip animation
    pub artifact_put_in_x: f64,
    pub artifact_put_in_y: f64,
    pub artifact_upgrade_x: f64,
    pub artifact_upgrade_y: f64,
    pub artifact_skip_x: f64,
    pub artifact_skip1_y: f64,
    pub artifact_skip2_y: f64,
    
    // 用于秘境挑战组队邀请
    pub online_challange_pos: Rect,
    pub online_challange_confirm_x: f64,
    pub online_challange_confirm_y: f64,

    // 用于UID的遮罩
    pub uid_pos: Rect,

    // 用于传送按钮区域
    pub tp_botton_pos: Rect,
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
        // 我寻思不是一样的嘛
        // w、h比例是一样的
        let convert_y = |y: f64| {
            y / self.height * h
        };
        PickupInfo { 
            left,
            top,
            width: w as u32,
            height: h as u32,

            f_area_position: convert_rect(&self.f_area_pos), 
            f_template_w: convert_x(self.f_template_w) as u32,
            f_template_h: convert_y(self.f_template_h) as u32,
            f_alpha_left: convert_x(self.f_area_left_alpha) as u32,

            pickup_x_beg: convert_x(self.pickup_x_beg) as u32,
            pickup_x_end: convert_x(self.pickup_x_end) as u32,

            pickup_y_gap: convert_y(self.pickup_y_gap) as u32,

            artifact_put_in_x: convert_x(self.artifact_put_in_x) as u32,
            artifact_put_in_y: convert_y(self.artifact_put_in_y) as u32,
            artifact_upgrade_x: convert_x(self.artifact_upgrade_x) as u32,
            artifact_upgrade_y: convert_y(self.artifact_upgrade_y) as u32,
            artifact_skip_x: convert_x(self.artifact_skip_x) as u32,
            artifact_skip1_y: convert_y(self.artifact_skip1_y) as u32,
            artifact_skip2_y: convert_y(self.artifact_skip2_y) as u32,

            online_challange_position: convert_rect(&self.online_challange_pos),
            online_challange_confirm_x: convert_x(self.online_challange_confirm_x) as u32,
            online_challange_confirm_y: convert_y(self.online_challange_confirm_y) as u32,

            uid_pos: convert_rect(&self.uid_pos),

            tp_botton_pos: convert_rect(&self.tp_botton_pos),
        }
    }
}

pub const WINDOW_16_9: WindowInfo = WindowInfo {
    width: 1920.0,
    height: 1080.0,

    f_area_pos: Rect(340.0, 1157.0, 720.0, 1090.0),
    f_template_w: 56.0,
    f_template_h: 38.0,
    f_area_left_alpha: 1140.0,
    
    pickup_x_beg: 1218.0,
    pickup_x_end: 1495.0,
    pickup_y_gap: 72.0,

    artifact_put_in_x: 1750.0,
    artifact_put_in_y: 770.0,
    artifact_upgrade_x: 1750.0,
    artifact_upgrade_y: 1020.0,
    artifact_skip_x: 128.0,
    artifact_skip1_y: 150.0,
    artifact_skip2_y: 220.0,

    online_challange_pos: Rect(46.0, 1031.0, 77.0, 831.0),
    online_challange_confirm_x: 1180.0,
    online_challange_confirm_y: 759.0,

    uid_pos: Rect(1051.0, 1865.0, 1073.0, 1682.0),

    tp_botton_pos: Rect(969.0, 1912.0, 1044.0, 1433.0),
};

pub const WINDOW_16_10: WindowInfo = WindowInfo {
    width: 2560.0,
    height: 1600.0,

    f_area_pos: Rect(502.0, 1540.0, 1057.0, 1454.0),
    f_template_w: 75.0,
    f_template_h: 50.0,
    f_area_left_alpha: 1522.0,
    
    pickup_x_beg: 1631.0,
    pickup_x_end: 1997.0,
    pickup_y_gap: 96.0, 

    // 草，并不支持快速强化，反正下版本就移除了
    artifact_put_in_x: 1750.0,
    artifact_put_in_y: 770.0,
    artifact_upgrade_x: 1750.0,
    artifact_upgrade_y: 1020.0,
    artifact_skip_x: 128.0,
    artifact_skip1_y: 150.0,
    artifact_skip2_y: 220.0,


    online_challange_pos: Rect(64.0, 1366.0, 100.0, 1154.0),
    online_challange_confirm_x: 1180.0,
    online_challange_confirm_y: 759.0,

    uid_pos: Rect(1583.0, 2508.0, 1600.0, 2209.0),

    // 也不支持快速传送。
    tp_botton_pos: Rect(969.0, 1912.0, 1044.0, 1433.0),
};