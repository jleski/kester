fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
    #[cfg(windows)]
    {
        embed_resource::compile("resources.rc", embed_resource::NONE);
    }
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }
}