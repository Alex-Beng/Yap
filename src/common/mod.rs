use std::io::stdin;
use std::process;
use std::ptr::null_mut;
use std::thread;
use std::time::Duration;

use crate::dto::GithubTag;

use log::error;
use reqwest::blocking::Client;
use reqwest::header::{HeaderValue, USER_AGENT};
use winapi::shared::minwindef::BOOL;
use winapi::shared::windef::HWND;
use winapi::um::securitybaseapi::{AllocateAndInitializeSid, CheckTokenMembership, FreeSid};
use winapi::um::winnt::{
    DOMAIN_ALIAS_RID_ADMINS, PSID, SECURITY_BUILTIN_DOMAIN_RID, SECURITY_NT_AUTHORITY,
    SID_IDENTIFIER_AUTHORITY,
};

#[derive(Clone, Copy)]
pub struct WindowHandle {
    pub hwnd: HWND,
}

impl WindowHandle {
    pub fn new(hwnd: HWND) -> Option<Self> {
        if hwnd.is_null() {
            None
        } else {
            Some(WindowHandle { hwnd })
        }
    }

    pub fn as_ptr(&self) -> HWND {
        self.hwnd
    }
}

pub fn error_and_quit(msg: &str) -> ! {
    error!("{}, 按Enter退出", msg);
    let mut s: String = String::new();
    let _ = stdin().read_line(&mut s);
    process::exit(0);
}

pub fn error_and_quit_no_input(msg: &str) -> ! {
    error!("{}, 即将退出", msg);
    sleep(1000);
    process::exit(0);
}

pub fn sleep(ms: u32) {
    let time = Duration::from_millis(ms as u64);
    thread::sleep(time);
}

// 管理员权限
unsafe fn is_admin_unsafe() -> bool {
    let mut authority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_NT_AUTHORITY,
    };
    let mut group: PSID = null_mut();
    let mut b = AllocateAndInitializeSid(
        &mut authority as *mut SID_IDENTIFIER_AUTHORITY,
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut group as *mut PSID,
    );
    if b != 0 {
        let r = CheckTokenMembership(null_mut(), group, &mut b as *mut BOOL);
        if r == 0 {
            b = 0;
        }
        FreeSid(group);
    }

    b != 0
}

pub fn is_admin() -> bool {
    unsafe { is_admin_unsafe() }
}

// 版本更新
static mut VERSION: String = String::new();
unsafe fn get_version_unsafe() -> String {
    if VERSION == String::new() {
        let s = include_str!("../../Cargo.toml");
        for line in s.lines() {
            if line.starts_with("version = ") {
                let temp = line.split('\"').collect::<Vec<_>>();
                let version = String::from(temp[temp.len() - 2]);
                VERSION = version;
            }
        }
    }

    VERSION.clone()
}

pub fn get_version() -> String {
    unsafe { get_version_unsafe() }
}

pub fn check_update() -> Option<String> {
    let client = Client::new();

    let resp = client
        .get("https://api.github.com/repos/Alex-Beng/Yap/tags")
        .timeout(Duration::from_secs(5))
        .header(USER_AGENT, HeaderValue::from_static("reqwest"))
        .send()
        .ok()?
        .json::<Vec<GithubTag>>()
        .ok()?;

    let latest = if resp.is_empty() {
        return None;
    } else {
        resp[0].name.clone()
    };
    let latest = &latest[1..];

    let latest_sem: semver::Version = semver::Version::parse(latest).unwrap();
    let current_sem: semver::Version = semver::Version::parse(&get_version()).unwrap();

    if latest_sem > current_sem {
        Some(String::from(latest))
    } else {
        None
    }
}
