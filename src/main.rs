use std::io::stdin;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use image::imageops::grayscale;
use image::{DynamicImage, ImageBuffer, Pixel};
use imageproc::template_matching;

use image::{open, GenericImage, GrayImage, Luma, Rgb, RgbImage};
use imageproc::definitions::Image;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::map::map_colors;
use imageproc::rect::Rect;
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use std::env;
use std::f32;
use std::fs;
use std::path::PathBuf;

struct TemplateMatchingArgs {
    input_path: PathBuf,
    output_dir: PathBuf,
    template_x: u32,
    template_y: u32,
    template_w: u32,
    template_h: u32,
}

impl TemplateMatchingArgs {
    fn parse(args: Vec<String>) -> TemplateMatchingArgs {
        if args.len() != 7 {
            panic!(
                r#"
Usage:

     cargo run --example template_matching input_path output_dir template_x template_y template_w template_h

Loads the image at input_path and extracts a region with the given location and size to use as the matching
template. Calls match_template on the input image and this template, and saves the results to output_dir.
"#
            );
        }

        let input_path = PathBuf::from(&args[1]);
        let output_dir = PathBuf::from(&args[2]);
        let template_x = args[3].parse().unwrap();
        let template_y = args[4].parse().unwrap();
        let template_w = args[5].parse().unwrap();
        let template_h = args[6].parse().unwrap();

        TemplateMatchingArgs {
            input_path,
            output_dir,
            template_x,
            template_y,
            template_w,
            template_h,
        }
    }
}

/// Convert an f32-valued image to a 8 bit depth, covering the whole
/// available intensity range.
fn convert_to_gray_image(image: &Image<Luma<f32>>) -> GrayImage {
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;

    for p in image.iter() {
        lo = if *p < lo { *p } else { lo };
        hi = if *p > hi { *p } else { hi };
    }

    let range = hi - lo;
    let scale = |x| (255.0 * (x - lo) / range) as u8;
    map_colors(image, |p| Luma([scale(p[0])]))
}

fn copy_sub_image(image: &GrayImage, x: u32, y: u32, w: u32, h: u32) -> GrayImage {
    // print!("{} {} {},{} {}/{}",x, y, x+w, y+h, image.width(), image.height());
    assert!(
        x + w < image.width() && y + h < image.height(),
        "invalid sub-image"
    );

    let mut result = GrayImage::new(w, h);
    for sy in 0..h {
        for sx in 0..w {
            result.put_pixel(sx, sy, *image.get_pixel(x + sx, y + sy));
        }
    }

    result
}

fn draw_green_rect(image: &GrayImage, rect: Rect) -> RgbImage {
    let mut color_image = map_colors(image, |p| Rgb([p[0], p[0], p[0]]));
    draw_hollow_rect_mut(&mut color_image, rect, Rgb([0, 255, 0]));
    color_image
}

fn run_match_template(
    args: &TemplateMatchingArgs,
    image: &GrayImage,
    template: &GrayImage,
    method: MatchTemplateMethod,
) -> RgbImage {
    // Match the template and convert to u8 depth to display
    let result = match_template(&image, &template, method);
    let result_scaled = convert_to_gray_image(&result);

    // Pad the result to the same size as the input image, to make them easier to compare
    let mut result_padded = GrayImage::new(image.width(), image.height());
    result_padded
        .copy_from(&result_scaled, args.template_w / 2, args.template_h / 2)
        .unwrap();

    // Show location the template was extracted from
    let roi = Rect::at(args.template_x as i32, args.template_y as i32)
        .of_size(args.template_w, args.template_h);

    draw_green_rect(&result_padded, roi)
}

fn open_local(path: String) -> DynamicImage {
    let img = image::open(path).unwrap();
    img
}

// possible area
// 1095, 340
// 1136, 720
fn main() {
    
    for i in 1..=7 {
        // 测试性能
        
        let filename = format!("pics/{}.jpg", i);
        let image: ImageBuffer<Luma<u8>, Vec<u8>> = open_local(filename).to_luma8();
        
        let start = Instant::now();

        let template: ImageBuffer<Luma<u8>, Vec<u8>> =
            open_local("pics/FFF.bmp".to_string()).to_luma8();
        // image.save("hhh.jpg");
        // template.save("hh.jpg");

        let pos_x: u32 = 1095;
        let pos_y: u32 = 340;
        let pos_w: u32 = 60;
        let pos_h: u32 = 380;
        // println!("wocao? {} {}", template.width(), template.height());
        let args = TemplateMatchingArgs {
            input_path: PathBuf::from(""),
            output_dir: PathBuf::from("pics/"),
            template_x: pos_x,
            template_y: pos_y,
            template_w: pos_w,
            template_h: pos_h,
        };
        // println!("wocao");
        let F_area = copy_sub_image(
            &image,
            args.template_x,
            args.template_y,
            args.template_w,
            args.template_h,
        );
        // println!("wocaonima {} {}", F_area.width(), F_area.height());
        // F_area.save("ccc.jpg");

        let result = match_template(&F_area, &template, MatchTemplateMethod::CrossCorrelation);
        let result_scaled: ImageBuffer<Luma<u8>, Vec<u8>> = convert_to_gray_image(&result);
        // result_scaled.save("wocao.jpg").unwrap();

        // println!("wocao {} {}", result.width(), result.height());
        // println!("wocao {} {}", result_scaled.width(), result_scaled.height());

        let res_mm = find_extremes(&result);
        // println!(
        //     "wocao {} {},{}",
        //     res_mm.min_value, res_mm.min_value_location.0, res_mm.min_value_location.1
        // );
        let res_x = res_mm.max_value_location.0 + pos_x;
        let res_y = res_mm.max_value_location.1 + pos_y;
        // let res_x = res_mm.max_value_location.0;
        // let res_y = res_mm.max_value_location.1;

        // result coor to F_area coor
        let roi = Rect::at(res_x as i32, res_y as i32).of_size(template.width(), template.height());
        
        let resname = format!("pics/res_{}.jpg", i);
        // draw_green_rect(&image, roi).save(resname);
        let duration = start.elapsed();
        println!("{} use: {:?}", i, duration);
        draw_green_rect(&image, roi).save(resname);
    }
}
