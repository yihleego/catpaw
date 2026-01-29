fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        embed_resource::compile("build/windows/icon.rc", embed_resource::NONE);
    }
    println!("cargo:rerun-if-changed=build/windows/icon.rc");
}
