use std::env;

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_ref().map(String::as_str) {
        Ok("macos") | Ok("ios") => println!("cargo:rustc-cfg=apple"),
        Ok("freebsd") | Ok("dragonfly") => println!("cargo:rustc-cfg=freebsdlike"),
        Ok("openbsd") | Ok("netbsd") | Ok("bitrig") => println!("cargo:rust-cfg=netbsdlike"),
        _ => ()
    }
}
