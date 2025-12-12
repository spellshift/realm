use std::env;

fn main() {
    let host_os = env::consts::OS;
    match host_os {
        "windows" => println!("cargo:rustc-cfg=host_family=\"windows\""),
        _ => println!("cargo:rustc-cfg=host_family=\"unix\""),
    }
}
