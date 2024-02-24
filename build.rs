extern crate winres;

fn main() {
    // only run if target os is windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
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

    let mut version = 0 as u64;
    version |= 0 << 48;
    version |= 2 << 32;
    version |= 2 << 16;
    version |= 0;

    res.set_version_info(VersionInfo::FILEVERSION, version)
        .set_version_info(VersionInfo::PRODUCTVERSION, version)
        .set_manifest_file("manifest.xml");

    if let Err(e) = res.compile() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    // }
}
