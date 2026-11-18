fn main() {
    // Windows: Mathéo's icon in the .exe.
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/matheo.ico");
        res.compile().expect("compile Windows resources");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/matheo.ico");
}
