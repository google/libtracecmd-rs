fn main() {
    match pkg_config::probe_library("libtracecmd") {
        Ok(_) => (),
        Err(e) => panic!("libtracecmd not found: {}", e),
    };
    println!("cargo:rustc-link-lib=dylib=tracecmd");
    println!("cargo:rustc-link-lib=dylib=tracefs");
    println!("cargo:rustc-link-lib=dylib=traceevent");
}
