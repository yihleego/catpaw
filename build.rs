fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        // 指向图标文件的路径 (相对于项目根目录)
        res.set_icon("build/windows/icon.ico");
        
        // 编译资源
        if let Err(e) = res.compile() {
            eprintln!("Error compiling Windows resources: {}", e);
            std::process::exit(1);
        }
    }
    
    // 监控图标文件变化
    println!("cargo:rerun-if-changed=build/windows/icon.ico");
}