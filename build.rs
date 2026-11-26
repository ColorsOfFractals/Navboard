fn main() {
    println!("cargo:rerun-if-changed=assets/Navboard_Icon.ico");

    if cfg!(target_os = "windows") {
        winres::WindowsResource::new()
            .set_icon("assets/Navboard_Icon.ico")
            .compile()
            .expect("Failed to embed Navboard icon");
    }
}