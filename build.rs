fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=resources");

    glib_build_tools::compile_resources(
        &["resources"],
        "resources/mview6.gresource.xml",
        "mview6.gresource",
    );

    if std::env::var("TARGET").expect("Unable to get TARGET") != "aarch64" {
        println!("cargo:rustc-cfg=feature=\"mupdf\"");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("resources/mview6.ico");
        res.compile().unwrap();
    }
}
