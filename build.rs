fn main() {
    let dst = cmake::build("libnotilus");
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=notilus");
    println!("cargo:rustc-link-lib=dylib=notify");
    println!("cargo:rustc-link-lib=dylib=gobject-2.0");
}
