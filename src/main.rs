use std::io::stdin;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};
use std::env;
use std::f32;
use std::fs;
use std::path::PathBuf;


use yap::capture;
use yap::common;
use yap::inference::img_process::rgb_to_l;
use yap::info;
use yap::pickupper::pickupper::Pickupper;

use image::imageops::grayscale;
use image::{DynamicImage, ImageBuffer, Pixel};
use imageproc::template_matching;

use image::{open, GenericImage, GrayImage, Luma, Rgb, RgbImage};
use imageproc::definitions::Image;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::map::map_colors;
use imageproc::rect::Rect;
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};

use winapi::um::winuser::{SetForegroundWindow, GetDpiForSystem, SetThreadDpiAwarenessContext, ShowWindow, SW_SHOW, SW_RESTORE, GetSystemMetrics, SetProcessDPIAware};

use clap::{Arg, App};

use env_logger::{Env, Builder, Target};
use log::{info, LevelFilter, warn};



fn main() {

    if !common::is_admin() {
        common::error_and_quit("请以管理员身份运行该程序");
    }

    if let Some(v) = common::check_update() {
        warn!("检测到新版本，请手动更新：{}", v);
    }
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
            .default_value("20")
            .help("一次检测推理拾取的间隔，单位ms"))
        .arg(Arg::with_name("template-threshold")
            .long("template-threshold")
            .short("t")
            .required(false)
            .takes_value(true)
            .default_value("0.1")
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
        .get_matches();
    
    let dump: bool = matches.is_present("dump");
    let dump_path = matches.value_of("dump").unwrap_or("./dumps/");
    let cnt:u32 = matches.value_of("dump_idx").unwrap_or("0").parse::<u32>().unwrap();
    let infer_gap: u32 = matches.value_of("infer_gap").unwrap_or("20").parse::<u32>().unwrap();
    let template_threshold: f32 = matches.value_of("template-threshold").unwrap_or("0.1").parse::<f32>().unwrap();
    let mut channal = matches.value_of("channal").unwrap_or("gray");
    let log_level = matches.value_of("log").unwrap_or("warn");
    let no_pickup = matches.is_present("no_pickup");

    // 首先更改日志等级
    let mut builder = Builder::from_env(Env::default().default_filter_or(log_level));
    builder.target(Target::Stdout);
    builder.init();


    
    // 检查dump_path是否存在，不存在则创建
    if dump && !Path::new(dump_path).exists() {
        fs::create_dir_all(dump_path).unwrap();
    }
    
    // 检查template threshold是否合法
    if template_threshold < 0.0  {
        common::error_and_quit("template threshold必须大于零");
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


    let hwnd = match capture::find_window("原神") {
        Err(s) => {
            common::error_and_quit("未找到原神窗口，请确认原神已经开启");
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
    } else {
        common::error_and_quit("不支持的分辨率");
    }

    // Pickup 主逻辑
    let mut pickupper = Pickupper::new(info, String::from("./black_lists.json"), use_l);

    pickupper.start(dump, dump_path.to_string(), cnt, infer_gap,  template_threshold, !no_pickup);


}
