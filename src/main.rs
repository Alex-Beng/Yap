use std::io::stdin;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};
use std::env;
use std::f32;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::ptr::null_mut;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

use enigo::MouseControllable;
use yap::capture;
use yap::common::{self, sleep};
use yap::inference::img_process::rgb_to_l;
use yap::info;
use yap::pickupper::pickupper::{Pickupper, PickupCofig};

use hotkey;
use rand::{self, Rng};

use image::imageops::grayscale;
use image::{DynamicImage, ImageBuffer, Pixel};
use imageproc::template_matching;

use image::{open, GenericImage, GrayImage, Luma, Rgb, RgbImage};
use imageproc::definitions::Image;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::map::map_colors;
use imageproc::rect::Rect;
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};

use winapi::um::winuser::{SetForegroundWindow, GetDpiForSystem, SetThreadDpiAwarenessContext, ShowWindow, SW_SHOW, SW_RESTORE, GetSystemMetrics, SetProcessDPIAware, GetDpiForWindow, SM_CXSCREEN, SM_CYSCREEN, SetTimer};
use winapi::um::shellscalingapi::SetProcessDpiAwareness;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{CreateSolidBrush, RGB, SetTextColor, CreateFontW};
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, GetDC, GetDesktopWindow, GetWindowRect, ReleaseDC,
    SetLayeredWindowAttributes, UpdateWindow, WS_EX_LAYERED, WS_EX_TOPMOST,
    WS_POPUP, WM_PAINT, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_NCHITTEST, WM_SIZE, WNDCLASSW,
    PAINTSTRUCT, RegisterClassW, LWA_ALPHA, MSG, GetMessageW, TranslateMessage, DispatchMessageW, BeginPaint, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DrawTextW, EndPaint, PostQuitMessage, HTCAPTION, InvalidateRect, LWA_COLORKEY, DT_CALCRECT, FillRect, GetWindowLongW, GWL_EXSTYLE, WS_EX_TOOLWINDOW, SetWindowLongW, SetWindowLongPtrW, WS_DISABLED, FindWindowW, SendMessageW, WS_EX_APPWINDOW,
     
};
use winapi::shared::windef::{RECT, HWND, HBRUSH, HDC, POINT};
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};


use clap::{Arg, App};

use env_logger::{Env, Builder, Target};
use log::{info, LevelFilter, warn};



fn main() {

    let version = common::get_version();

    let matches = App::new("YAP - 原神自动拾取器")
        .author("Alex-Beng <pc98@qq.com>")
        .about("Genshin Impact Pickup Helper")
        .version(version.as_str())
        .arg(Arg::with_name("dump")
            .long("dump")
            .required(false)
            .takes_value(true)
            .help("输出模型预测结果、原始图像、二值图像至指定的文件夹，debug专用"))
        .arg(Arg::with_name("dump_idx")
            .long("dump-idx")
            .short("i")
            .required(false)
            .takes_value(true)
            .default_value("0")
            .help("执行dump时，输出结果起始的index"))
        .arg(Arg::with_name("infer_gap")
            .long("infer-gap")
            .short("g")
            .required(false)
            .takes_value(true)
            .default_value("0")
            .help("一次检测推理拾取的间隔，单位ms"))
        .arg(Arg::with_name("template-threshold")
            .long("template-threshold")
            .short("t")
            .required(false)
            .takes_value(true)
            .default_value("0.08")
            .help("模板匹配的阈值，约小越严格，灰度通道中匹配值在0.01-0.09左右"))
        .arg(Arg::with_name("channal")
            .long("channal")
            .short("c")
            .required(false)
            .takes_value(true)
            .default_value("gray")
            .help("模板匹配时使用的通道，默认使用gray通道，另一个可选值为L*，推荐匹配阈值固定为0.01"))
        .arg(Arg::with_name("log")
            .long("log")
            .required(false)
            .takes_value(true)
            .default_value("warn")
            .help("日志等级，可选值为trace, debug, info, warn, error"))
        .arg(Arg::with_name("no_pickup")
            .long("no-pickup")
            .required(false)
            .takes_value(false)
            .help("不执行拾取，仅info拾取动作，debug专用"))
        .arg(Arg::with_name("hotkey")
            .required(false)
            .takes_value(false)
            .help("是否注册hotkey用于调整拾取时序，debug专用"))
        .arg(Arg::with_name("click_tp")
            .required(false)
            .takes_value(false)
            .help("是否自动点击传送"))
        .get_matches();
    
    let dump: bool = matches.is_present("dump");
    let dump_path = matches.value_of("dump").unwrap_or("./dumps/");
    let cnt:u32 = matches.value_of("dump_idx").unwrap_or("0").parse::<u32>().unwrap();
    let infer_gap: u32 = matches.value_of("infer_gap").unwrap_or("0").parse::<u32>().unwrap();
    let template_threshold: f32 = matches.value_of("template-threshold").unwrap_or("0.08").parse::<f32>().unwrap();
    let mut channal = matches.value_of("channal").unwrap_or("gray");
    let log_level = matches.value_of("log").unwrap_or("warn");
    let no_pickup = matches.is_present("no_pickup");
    let reg_hotkey = matches.is_present("hotkey");
    let click_tp = matches.is_present("click_tp");

    // 首先更改日志等级
    let mut builder = Builder::from_env(Env::default().default_filter_or(log_level));
    builder.target(Target::Stdout);
    builder.init();

    // 移动管理员权限检查、版本更新至logger初始化之后
    if !common::is_admin() {
        common::error_and_quit_no_input("请以管理员身份运行该程序");
    }

    // 设置 DPI 感知级别
    // 用于适配高DPI屏幕
    let dpi_awareness = unsafe { SetProcessDpiAwareness(2) };
    if dpi_awareness != 0 {
        warn!("SetProcessDpiAwareness failed，高DPI可能导致程序无法正常运行");
    }

    if let Some(v) = common::check_update() {
        warn!("检测到新版本，请手动更新：{}", v);
    }
    
    // 尝试从可能存在的 config.json 读取可能存在的拾取参数
    let mut infer_gap_default = infer_gap;
    let mut f_internal_default = 50;
    let mut f_gap_default = 85;
    let mut scroll_gap_default = 70;
    let mut click_tp_default = false;
    let config_path = Path::new("./config.json");
    if config_path.exists() {
        let config = fs::read_to_string(config_path).unwrap();
        let config: serde_json::Value = serde_json::from_str(&config).unwrap();
        // 拾取参数是可能存在，需要逐一检查
        if let Some(infer_gap) = config.get("infer_gap") {
            if infer_gap.is_u64() {
                infer_gap_default = infer_gap.as_u64().unwrap() as u32;
            }
        }
        if let Some(f_internal) = config.get("f_internal") {
            if f_internal.is_u64() {
                f_internal_default = f_internal.as_u64().unwrap() as u32;
            }
        }
        if let Some(f_gap) = config.get("f_gap") {
            if f_gap.is_u64() {
                f_gap_default = f_gap.as_u64().unwrap() as u32;
            }
        }
        if let Some(scroll_gap) = config.get("scroll_gap") {
            if scroll_gap.is_u64() {
                scroll_gap_default = scroll_gap.as_u64().unwrap() as u32;
            }
        }
        if let Some(click_tp) = config.get("click_tp") {
            if click_tp.is_boolean() {
                click_tp_default = click_tp.as_bool().unwrap();
            }
        }
    }

    // 检查dump_path是否存在，不存在则创建
    if dump && !Path::new(dump_path).exists() {
        fs::create_dir_all(dump_path).unwrap();
    }
    
    // 检查template threshold是否合法
    if template_threshold < 0.0  {
        common::error_and_quit_no_input("template threshold必须大于零");
    }
    
    if channal != "L*" && channal != "gray" {
        // common::error_and_quit("channal参数必须为L*或gray");
        warn!("channal参数必须为L*或gray，使用默认值gray");
        channal = "gray";
    }

    let mut use_l = false;
    if channal == "L*" {
        use_l = true;
        info!("使用L*通道进行模板匹配");
        // template_threshold = 0.01;
    }

    let do_pickup_signal = Arc::new(Mutex::new(!no_pickup));
    let infer_gap_signal = Arc::new(RwLock::new(infer_gap_default));
    let f_inter_signal = Arc::new(RwLock::new(f_internal_default));
    let f_gap_signal = Arc::new(RwLock::new(f_gap_default));
    let scroll_gap_signal = Arc::new(RwLock::new(scroll_gap_default));
    let click_tp_signal = Arc::new(Mutex::new(click_tp_default));

    let mut is_cloud = false;
    let hwnd = match capture::find_window_local() {
        Err(_) => {
            warn!("未找到原神窗口，尝试寻找云·原神");
            match capture::find_window_cloud() {
                Ok(h) => {
                    is_cloud = true;
                    h
                },
                Err(_) => {
                    common::error_and_quit_no_input("未找到原神窗口，请确认原神已经开启");
                }
            }
        },
        Ok(h) => h,
    };
    

    unsafe { ShowWindow(hwnd, SW_RESTORE); }
    unsafe { SetForegroundWindow(hwnd); }
    common::sleep(1000);

    // get windows size
    let rect = capture::get_client_rect(hwnd).unwrap();
    info!("left = {}, top = {}, width = {}, height = {}", rect.left, rect.top, rect.width, rect.height);
    
    capture::capture_absolute_image(&rect).unwrap().save("test.png").unwrap();


    let info: info::PickupInfo;
    if rect.height * 16 == rect.width * 9 {
        info = info::PickupInfo::from_16_9(rect.width as u32, rect.height as u32, rect.left, rect.top);
    } 
    else if rect.height * 16 == rect.width * 10 {
        info = info::PickupInfo::from_16_10(rect.width as u32, rect.height as u32, rect.left, rect.top);
    } 
    else {
        common::error_and_quit_no_input("不支持的分辨率");
    }

    // 添加监听的hotkey
    let do_pickup_signal_clone = do_pickup_signal.clone();

    let f_inter_signal_clone = f_inter_signal.clone();
    let f_inter_signal_clone_d = f_inter_signal.clone();
    
    let f_gap_signal_clone = f_gap_signal.clone();
    let f_gap_signal_clone_d = f_gap_signal.clone();
    
    let infer_gap_signal_clone = infer_gap_signal.clone();
    let infer_gap_signal_clone_d = infer_gap_signal.clone();

    let scroll_gap_signal_clone = scroll_gap_signal.clone();
    let scroll_gap_signal_clone_d = scroll_gap_signal.clone();

    let click_tp_signal_clone = click_tp_signal.clone();

    let info_for_artifacts = info.clone();
    

    let pk_config = PickupCofig {
        info,
        bw_path: String::from("."),
        use_l,
        dump,
        dump_path: dump_path.to_string(),
        dump_cnt: cnt,
        temp_thre: template_threshold,
        do_pickup: do_pickup_signal,
        infer_gap: infer_gap_signal,
        f_inter: f_inter_signal,
        f_gap: f_gap_signal,
        scroll_gap: scroll_gap_signal,
        click_tp: click_tp_signal,
    };
    let uid_pos = pk_config.info.uid_pos.clone();
    let uid_pos = RECT {
        left: uid_pos.left + pk_config.info.left,
        top: uid_pos.top + pk_config.info.top,
        right: uid_pos.right + pk_config.info.left,
        bottom: uid_pos.bottom + pk_config.info.top,
    };
    // println!("uid_pos: {}, {}, {}, {}", uid_pos.left, uid_pos.top, uid_pos.right, uid_pos.bottom);

    // 三个子线程+主线程pickupper
    // 由于主线程是while true，所以不需要join

    // 创建UID的遮罩窗口
    let uid_handle = std::thread::spawn(move || {
        // 注册窗口类
        let class_name = "FloatingWindowClass".to_string();
        let class_name2 = "FloatingWindowClass".to_string();
        let h_instance = unsafe { GetModuleHandleW(null_mut()) };
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            hInstance: h_instance,
            lpszClassName: capture::encode_wide(class_name).as_ptr(),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };
        unsafe { RegisterClassW(&wnd_class) };

        // 创建窗口
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST,
                capture::encode_wide(class_name2).as_ptr(),
                capture::encode_wide("YAP float window".to_string()).as_ptr(),
                WS_POPUP,
                uid_pos.left,
                uid_pos.top,
                uid_pos.right - uid_pos.left,
                uid_pos.bottom - uid_pos.top,
                null_mut(),
                null_mut(),
                h_instance,
                null_mut(),
            )
        };

        // 设置不在任务栏显示
        unsafe {
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            SetWindowLongW(hwnd, GWL_EXSTYLE,  ex_style | WS_EX_TOOLWINDOW as i32 | WS_EX_LAYERED as i32);
        }


        // 设置窗口透明度
        unsafe { SetLayeredWindowAttributes(hwnd, 0, 200, LWA_COLORKEY) };

        // 显示窗口
        unsafe { ShowWindow(hwnd, SW_SHOW) };
        unsafe { UpdateWindow(hwnd) };
        
        let hwnd = match capture::find_window_local() {
            Err(_) => {
                // warn!("未找到原神窗口，尝试寻找云·原神");
                match capture::find_window_cloud() {
                    Ok(h) => {
                        h
                    },
                    Err(_) => {
                        common::error_and_quit_no_input("未找到原神窗口，请确认原神已经开启");
                    }
                }
            },
            Ok(h) => h,
        };
        
        unsafe { ShowWindow(hwnd, SW_RESTORE); }
        unsafe { SetForegroundWindow(hwnd); }
        common::sleep(1000);

        // 消息循环
        let mut msg = MSG {
            hwnd: null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT { x: 0, y: 0 },
        };
        while unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) } > 0 {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });


    // 监听快捷键
    let hotkey_handle = std::thread::spawn(move || {
        let mut hk = hotkey::Listener::new();
        let do_pk_signal: Arc<Mutex<bool>> = do_pickup_signal_clone;
        let click_tp_signal_clone: Arc<Mutex<bool>> = click_tp_signal_clone;
        if reg_hotkey {

            
            let infer_gap_signal= infer_gap_signal_clone;
            let infer_gap_signal_d = infer_gap_signal_clone_d;
            // ALT + J 
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'J' as u32, 
                move || {
                    let mut signal = infer_gap_signal.write().unwrap();
                    warn!("ALT + J with {}", *signal);
                    // let mut signal = infer_gap_signal.write().unwrap();
                    *signal = *signal + 1;
                    warn!("ALT + J 增加 infer gap 至 {} ms", *signal);
                    
                    
                }
            ).unwrap();
            // ALT + K
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'K' as u32, 
                move || {
                    let mut signal = infer_gap_signal_d.write().unwrap();
                    warn!("ALT + K with {}", *signal);
                    if *signal == 0 {
                        return;
                    }
                    *signal -= 1;
                    warn!("ALT + K 减小 infer gap 至 {} ms", *signal);
                    
                }
            ).unwrap();

            let f_inter_signal = f_inter_signal_clone;
            let f_inter_signal_d = f_inter_signal_clone_d;
            // ALT + U
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'U' as u32, 
                move || {
                    let mut signal = f_inter_signal.write().unwrap();
                    warn!("ALT + U with {}", *signal);
                    *signal += 1;
                    warn!("ALT + U 增加 f_inter 至 {} ms", *signal);
                    
                }
            ).unwrap();
            // ALT + I
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'I' as u32, 
                move || {
                    let mut signal = f_inter_signal_d.write().unwrap();
                    warn!("ALT + I with {}", *signal);
                    if *signal == 0 {
                        return;
                    }
                    *signal -= 1;
                    warn!("ALT + I 减小 f_inter 至 {} ms", *signal);
                    
                }
            ).unwrap();

            let f_gap_signal = f_gap_signal_clone;
            let f_gap_signal_d = f_gap_signal_clone_d;
            // ALT + L
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'L' as u32, 
                move || {
                    let mut signal = f_gap_signal.write().unwrap();
                    warn!("ALT + L with {}", *signal);
                    *signal += 1;
                    warn!("ALT + L 增加 f_gap 至 {} ms", *signal);
                    
                }
            ).unwrap();
            // ALT + H
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'H' as u32, 
                move || {
                    let mut signal = f_gap_signal_d.write().unwrap();
                    warn!("ALT + H with {}", *signal);
                    if *signal == 0 {
                        return;
                    }
                    *signal -= 1;
                    warn!("ALT + H 减小 f_gap 至 {} ms", *signal);
                    
                }
            ).unwrap();

            let scroll_gap_signal = scroll_gap_signal_clone;
            let scroll_gap_signal_d = scroll_gap_signal_clone_d;
            // ALT + O
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'O' as u32, 
                move || {
                    let mut signal = scroll_gap_signal.write().unwrap();
                    warn!("ALT + O with {}", *signal);
                    *signal += 1;
                    warn!("ALT + O 增加 scroll_gap 至 {} ms", *signal);
                    
                }
            ).unwrap();
            // ALT + P
            hk.register_hotkey(
                hotkey::modifiers::ALT,
                'P' as u32, 
                move || {
                    let mut signal = scroll_gap_signal_d.write().unwrap();
                    warn!("ALT + P with {}", *signal);
                    if *signal == 0 {
                        return;
                    }
                    *signal -= 1;
                    warn!("ALT + P 减小 scroll_gap 至 {} ms", *signal);
                    
                }
            ).unwrap();
        }

        // 保留切换、强化、click tp
        // ALT + 0
        hk.register_hotkey(
            hotkey::modifiers::ALT,
            '0' as u32, 
            move || {
                let mut signal = do_pk_signal.lock().unwrap();
                *signal = !*signal;
                warn!("ALT + 0 切换 pickup 模式为 {}", *signal);              
            }
        ).unwrap();

        // ALT + B
        // 快速强化圣遗物
        hk.register_hotkey(
            hotkey::modifiers::ALT,
            'B' as u32, 
            move || {
                warn!("ALT + B 固定动作强化圣遗物");

                let mut enigo = enigo::Enigo::new();

                // do once
                enigo.mouse_move_to(  
                    info_for_artifacts.artifact_put_in_x as i32 + info_for_artifacts.left as i32, 
                    info_for_artifacts.artifact_put_in_y as i32 + info_for_artifacts.top as i32
                );
                enigo.mouse_click(enigo::MouseButton::Left);
                sleep(100);
                // warn!("click {}, {}", info_for_artifacts.artifact_put_in_x, info_for_artifacts.artifact_put_in_y);

                enigo.mouse_move_to(
                    info_for_artifacts.artifact_upgrade_x as i32 + info_for_artifacts.left as i32,
                    info_for_artifacts.artifact_upgrade_y as i32 + info_for_artifacts.top as i32
                );
                enigo.mouse_click(enigo::MouseButton::Left);
                sleep(100);

                enigo.mouse_move_to(
                    info_for_artifacts.artifact_skip_x as i32 + info_for_artifacts.left as i32,
                    info_for_artifacts.artifact_skip1_y as i32 + info_for_artifacts.top as i32
                );
                enigo.mouse_click(enigo::MouseButton::Left);
                sleep(100);

                enigo.mouse_move_to(
                    info_for_artifacts.artifact_skip_x as i32 + info_for_artifacts.left as i32,
                    info_for_artifacts.artifact_skip2_y as i32 + info_for_artifacts.top as i32
                );
                enigo.mouse_click(enigo::MouseButton::Left);
                sleep(100);

                enigo.mouse_move_to(  
                    info_for_artifacts.artifact_put_in_x as i32 + info_for_artifacts.left as i32, 
                    info_for_artifacts.artifact_put_in_y as i32 + info_for_artifacts.top as i32
                );
            }
        ).unwrap();

        // ALT + 9
        // 切换click tp
        hk.register_hotkey(
            hotkey::modifiers::ALT,
            '9' as u32, 
            move || {
                let mut signal = click_tp_signal_clone.lock().unwrap();
                *signal = !*signal;
                warn!("ALT + 9 切换 click tp 模式为 {}", *signal);              
            }
        ).unwrap();

        hk.listen();
    });
    // 监视原神进程是否结束
    let monitor_handel = std::thread::spawn(move ||{
        loop {
            if is_cloud {
                let hwnd = match capture::find_window_cloud() {
                    Ok(h) => h,
                    Err(_) => {
                        common::error_and_quit_no_input("原神进程已结束");
                    }
                };
            } else {
                let hwnd = match capture::find_window_local() {
                    Ok(h) => h,
                    Err(_) => {
                        common::error_and_quit_no_input("原神进程已结束");
                    }
                };
            }
            sleep(5000);
        }
    });
    
    let mut pickupper = Pickupper::new(pk_config);
    pickupper.start();


}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rect = RECT { 
                left: 0,
                top: 0,
                right: 400,
                bottom: 400,
            };
            // 在多个字符串中随机一个显示。
            let strs: Vec<&str> = vec![
                "UID: 1145141919810", 
                "UID: 7777777777777",
                "UID: 乐乐乐乐乐乐乐乐乐"
            ];
            let rand_idx = rand::thread_rng().gen_range(0..strs.len());
            let text = capture::encode_wide(strs[rand_idx].to_string());

            let white_brush: HBRUSH = CreateSolidBrush(RGB(255, 255, 255));
            FillRect(hdc, &mut rect as *mut RECT, white_brush);

            // println!("rect: {}, {}, {}, {}", rect.left, rect.top, rect.right, rect.bottom);
            // use DT_CALCRECT to get the rect
            DrawTextW(hdc, text.as_ptr(), -1, &mut rect as *mut RECT,  DT_CALCRECT);
            // println!("rect: {}, {}, {}, {}", rect.left, rect.top, rect.right, rect.bottom);
            SetTextColor(hdc, RGB(1, 0, 0));
            DrawTextW(hdc, text.as_ptr(), -1, &mut rect as *mut RECT,  DT_SINGLELINE);
            EndPaint(hwnd, &ps);

            ReleaseDC(hwnd, hdc);            
            0
        }
        WM_CLOSE | WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_ERASEBKGND => 1,
        WM_NCHITTEST => HTCAPTION,
        WM_SIZE => {
            let rect = RECT { 
                left: 0,
                top: 0,
                right: 400,
                bottom: 400,
            };
            InvalidateRect(hwnd, &rect, 1);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}
