#[cfg(windows)] extern crate winapi;
use std::io::Error;
use std::ffi::{OsStr};
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::mem::{size_of, transmute};

use winapi::um::winuser::{
    FindWindowW,
    GetDC,
    ReleaseDC,
    SetThreadDpiAwarenessContext,
    GetClientRect,
    SetForegroundWindow,
    ClientToScreen
};
use winapi::shared::windef::{HWND, RECT as WinRect, POINT as WinPoint, HDC, HBITMAP};
use winapi::shared::ntdef::NULL;
use winapi::um::wingdi::{
    CreateCompatibleDC,
    DeleteObject,
    BitBlt,
    SRCCOPY,
    CreateCompatibleBitmap,
    SelectObject,
    GetObjectW,
    BITMAP,
    BITMAPINFOHEADER,
    BI_RGB,
    GetDIBits,
    BITMAPINFO,
    DIB_RGB_COLORS,
};
use winapi::ctypes::{c_void};
use winapi::um::winbase::{GlobalAlloc, GHND, GlobalLock};

use image::{ImageBuffer, GrayImage};

use winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE;
use self::winapi::um::wingdi::{GetDeviceCaps, HORZRES};
use self::winapi::shared::windef::DPI_AWARENESS_CONTEXT_SYSTEM_AWARE;


#[derive(Debug)]
pub struct PixelRect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug)]
pub struct PixelRectBound {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub struct RawImage {
    pub data: Vec<f32>,
    pub w: u32,
    pub h: u32,
}

pub struct RawCaptureImage {
    pub data: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

#[inline]
fn get_index(width: u32, x: u32, y: u32) -> usize {
    (y * width + x) as usize
}

pub fn raw_to_img(im: &RawImage) -> GrayImage {
    let width = im.w;
    let height = im.h;
    let data = &im.data;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let index = get_index(width, x, y);
        let p = data[index];
        let pixel = (p * 255.0) as u32;
        let pixel: u8 = if pixel > 255 {
            255
        } else if pixel < 0 {
            0
        } else {
            pixel as u8
        };
        image::Luma([pixel])
    });

    img
}

pub fn uint8_raw_to_img(im: &RawImage) -> GrayImage {
    let width = im.w;
    let height = im.h;
    let data = &im.data;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let index = get_index(width, x, y);
        let pixel =  data[index] as u32;
        let pixel: u8 = if pixel > 255 {
            255
        } else if pixel < 0 {
            0
        } else {
            pixel as u8
        };
        image::Luma([pixel])
    });

    img
}


impl RawImage {
    pub fn to_gray_image(&self) -> GrayImage {
        raw_to_img(&self)
    }

    pub fn grayscale_to_gray_image(&self) -> GrayImage {
        uint8_raw_to_img(&self)
    }
}


impl RawCaptureImage {
    pub fn save(&self, path: &str) {
        let width = self.w;
        let height = self.h;
        let data = &self.data;

        let img = ImageBuffer::from_fn(width, height, |x, y| {
            let index = (y * self.w + x) as usize;

            let b = data[index * 4];
            let g = data[index * 4 + 1];
            let r = data[index * 4 + 2];

            image::Rgb([r, g, b])
            // image::Luma([pixel])
        });

        img.save(path);
    }

    pub fn to_RawImage(&self) -> RawImage {
        // let now = SystemTime::now();
        let vol = self.w * self.h;
        let mut data = vec![0.0; vol as usize];
        for i in 0..self.w as i32 {
            for j in 0..self.h as i32 {
                let x = i;
                let y = self.h as i32 - j - 1;
                let b: u8 = self.data[((y * self.w as i32 + x) * 4) as usize];
                let g: u8 = self.data[((y * self.w as i32 + x) * 4 + 1) as usize];
                let r: u8 = self.data[((y * self.w as i32 + x) * 4 + 2) as usize];

                let gray = r as f32 * 0.2989 + g as f32 * 0.5870 + b as f32 * 0.1140;
                let new_index = (j * self.w as i32 + i) as usize;
                data[new_index] = gray;
            }
        }

        let im = RawImage {
            data,
            w: self.w,
            h: self.h
        };
        // let im = pre_process(im);
        // No preprocess!

        // info!("preprocess time: {}ms", now.elapsed().unwrap().as_millis());
        // im.to_gray_image().save("test.png");
        im
    }
    
    pub fn crop_to_raw_img(&self, rect: &PixelRect) -> RawImage {
        // let now = SystemTime::now();
        let vol = rect.width * rect.height;
        let mut data = vec![0.0; vol as usize];
        for i in rect.left..rect.left + rect.width {
            for j in rect.top..rect.top + rect.height {
                let x = i;
                let y = self.h as i32 - j - 1;
                let b: u8 = self.data[((y * self.w as i32 + x) * 4) as usize];
                let g: u8 = self.data[((y * self.w as i32 + x) * 4 + 1) as usize];
                let r: u8 = self.data[((y * self.w as i32 + x) * 4 + 2) as usize];

                let gray = r as f32 * 0.2989 + g as f32 * 0.5870 + b as f32 * 0.1140;
                let new_index = ((j - rect.top) * rect.width + i - rect.left) as usize;
                data[new_index] = gray;
            }
        }

        let im = RawImage {
            data,
            w: rect.width as u32,
            h: rect.height as u32,
        };
        // let im = pre_process(im);
        // No preprocess!

        // info!("preprocess time: {}ms", now.elapsed().unwrap().as_millis());
        // im.to_gray_image().save("test.png");
        im
    }

}


// find_window -> capture 
// capture -> rgb/gray 


pub fn encode_wide(s: String) -> Vec<u16> {
    let wide: Vec<u16> = OsStr::new(&s).encode_wide().chain(once(0)).collect();
    wide
}

pub fn find_window_local() -> Result<HWND, String> {
    let wide = encode_wide(String::from("原神"));
    let class = encode_wide(String::from("UnityWndClass"));
    let result: HWND = unsafe {
        FindWindowW(class.as_ptr(), wide.as_ptr())
    };
    if result.is_null() {
        Err(String::from("cannot find window"))
    } else {
        Ok(result)
    }
}

pub fn find_window_cloud() -> Result<HWND, String> {
    let wide = encode_wide(String::from("云·原神"));
    // TODO: 云·原神
    let result: HWND = unsafe {
        FindWindowW(null_mut(), wide.as_ptr())
    };
    if result.is_null() {
        Err(String::from("cannot find window"))
    } else {
        Ok(result)
    }
}

// 获取窗口的尺寸
pub fn get_client_rect(hwnd: HWND) -> Result<PixelRect, String> {
    unsafe {
        get_client_rect_unsafe(hwnd)
    }
}

unsafe fn get_client_rect_unsafe(hwnd: HWND) -> Result<PixelRect, String> {
    let mut rect: WinRect = WinRect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    GetClientRect(hwnd, &mut rect);
    let width: i32 = rect.right;
    let height: i32 = rect.bottom;

    let mut point: WinPoint = WinPoint {
        x: 0,
        y: 0,
    };
    ClientToScreen(hwnd, &mut point as *mut WinPoint);
    let left: i32 = point.x;
    let top: i32 = point.y;

    Ok(PixelRect {
        left, top,
        width, height
    })
}

#[cfg(windows)]
unsafe fn unsafe_capture(rect: &PixelRect) -> Result<Vec<u8>, String> {
    // 这个函数用于设置当前线程的 DPI 感知级别，
    // DPI_AWARENESS_CONTEXT_SYSTEM_AWARE 表示系统 DPI 感知级别，即不会自动缩放应用程序的界面。
    // SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE);

    let dc_window: HDC = GetDC(null_mut());
    
    
    let dc_mem: HDC = CreateCompatibleDC(dc_window);
    if dc_mem.is_null() {
        return Err(String::from("CreateCompatibleDC Failed"));
    }

    let hbm: HBITMAP = CreateCompatibleBitmap(dc_window, rect.width, rect.height);
    if hbm.is_null() {
        return Err(String::from("CreateCompatibleBitmap failed"));
    }

    SelectObject(dc_mem, hbm as *mut c_void);

    let result = BitBlt(
        dc_mem,
        0,
        0,
        rect.width,
        rect.height,
        dc_window,
        rect.left,
        rect.top,
        SRCCOPY
    );
    if result == 0 {
        return Err(String::from("BitBlt failed"));
    }

    let mut bitmap: BITMAP = BITMAP {
        bmBits: 0 as *mut c_void,
        bmBitsPixel: 0,
        bmPlanes: 0,
        bmWidthBytes: 0,
        bmHeight: 0,
        bmWidth: 0,
        bmType: 0,
    };
    GetObjectW(
        hbm as *mut c_void,
        size_of::<BITMAP>() as i32,
        (&mut bitmap) as *mut BITMAP as *mut c_void
    );
    // println!("bitmap width: {}", bitmap.bmWidth);
    // println!("bitmap height: {}", bitmap.bmHeight);
    // println!("bitmap bits pixel: {}", bitmap.bmBitsPixel);

    let mut bi: BITMAPINFOHEADER = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: bitmap.bmWidth,
        biHeight: bitmap.bmHeight,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    let bitmap_size: usize = (((bitmap.bmWidth * 32 + 31) / 32) * 4 * bitmap.bmHeight) as usize;
    // println!("bitmap size: {}", bitmap_size);
    // let mut buffer: Vec<u8> = vec![0; bitmap_size];

    // let h_dib = GlobalAlloc(GHND, bitmap_size);
    // let lpbitmap = GlobalLock(h_dib);
    // println!("bitmap {:p}", lpbitmap);
    let mut buffer: Vec<u8> = vec![0; bitmap_size];

    GetDIBits(
        dc_window,
        hbm,
        0,
        bitmap.bmHeight as u32,
        // lpbitmap,
        buffer.as_mut_ptr() as *mut c_void,
        (&mut bi) as *mut BITMAPINFOHEADER as *mut BITMAPINFO,
        DIB_RGB_COLORS
    );

    // let buffer: Vec<u8> = Vec::from_raw_parts(lpbitmap as *mut u8, bitmap_size, bitmap_size);
    // for i in 0..10 {
    //     println!("{}", buffer[i]);
    // }

    // println!("{}", buffer[0]);

    DeleteObject(hbm as *mut c_void);
    DeleteObject(dc_mem as *mut c_void);
    ReleaseDC(null_mut(), dc_window);

    Ok(buffer)
}

#[cfg(windows)]
pub fn capture_absolute(rect: &PixelRect) -> Result<Vec<u8>, String> {
    unsafe {
        unsafe_capture(&rect)
    }
}

#[cfg(windows)]
pub fn capture_absolute_image(rect: &PixelRect) -> Result<image::RgbImage, String> {
    let raw: Vec<u8> = match capture_absolute(rect) {
        Err(s) => {
            return Err(s);
        },
        Ok(v) => v,
    };

    let height = rect.height as u32;
    let width = rect.width as u32;

    let mut img: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(
        width,
        height,
        move |x, y| {
            let y = height - y - 1;
            let b = raw[((y * width + x) * 4 + 0) as usize];
            let g = raw[((y * width + x) * 4 + 1) as usize];
            let r = raw[((y * width + x) * 4 + 2) as usize];
            image::Rgb([r, g, b])
        }
    );

    Ok(img)
}
