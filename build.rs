use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    println!("cargo:rustc-check-cfg=cfg(thumb1, thumb2)");

    if target.contains("thumbv7")
        || target.contains("thumbv8m.main")
        || target.contains("thumbv8r")
        || target.contains("armv7")
        || target.contains("armv8")
        || target.contains("armv9")
    {
        println!("cargo:rustc-cfg=thumb2");
    } else if target.contains("thumbv4")
        || target.contains("thumbv5")
        || target.contains("thumbv6")
        || target.contains("thumbv8m.base")
        || target.contains("armv4")
        || target.contains("armv5")
        || target.contains("armv6")
    {
        println!("cargo:rustc-cfg=thumb1");
    }
}
