use std::env;
use std::fs;
use std::path::PathBuf;
extern crate winres;
use winres::VersionInfo;

fn main() {
    // only run if target os is windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        println!("cargo:warning={:#?}", "This build script is only for windows target, skipping...");
        return;
    }
    for (key, value) in env::vars() {
        println!("cargo:ingo={}: {}", key, value);
    }

    // 获取profile env
    let profile = env::var("PROFILE").unwrap();
    
    let out_dir = "target".to_string() + "/" + &profile;
    let _ = fs::create_dir_all(&out_dir);

    println!("Output directory: {:?}", out_dir);

    println!("cargo::rerun-if-changed=models/model_training.onnx");
    println!("cargo::rerun-if-changed=models/index_2_word.json");
    println!("cargo::rerun-if-changed=config.json");
    let files_to_copy = vec![
        "models/model_training.onnx", 
        "models/index_2_word.json",
        // "config.json"
        ];

    // 复制每个文件到可执行文件输出目录
    for file in files_to_copy {
        let file_path = PathBuf::from(file);
        let file_name = file_path.file_name().unwrap();
        let dest_path = PathBuf::from(&out_dir).join(file_name);

        let _ = fs::copy(&file_path, &dest_path).unwrap();
    }

    let mut res = winres::WindowsResource::new();
    // if cfg!(unix) {
    //     // paths for X64 on archlinux
    //     res.set_toolkit_path("/usr/x86_64-w64-mingw32/bin");
    //     // ar tool for mingw in toolkit path
    //     res.set_ar_path("ar");
    //     // windres tool
    //     res.set_windres_path("/usr/bin/x86_64-w64-mingw32-windres");
    // }

    let mut version = 0;
    version |= 5 << 48;
    version |= 3 << 32;
    version |= 0 << 16;
    version |= 2;

    res.set_version_info(VersionInfo::FILEVERSION, version)
        .set_version_info(VersionInfo::PRODUCTVERSION, version)
        .set_manifest_file("manifest.xml");

    if let Err(e) = res.compile() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    // }
}
