#[cfg(target_os="windows")]
const HOST_OS: &str = "windows";

#[cfg(target_os="linux")]
const HOST_OS: &str = "linux";

#[cfg(target_os="macos")]
const HOST_OS: &str = "macos";

#[cfg(target_os="bsd")]
const HOST_OS: &str = "bsd";

fn set_host_os() {
    println!("cargo::rustc-check-cfg=cfg(host_os, values(\"linux\", \"windows\", \"macos\", \"bsd\"))");
    println!("cargo:rustc-cfg=host_os=\"{}\"", HOST_FAMILY);
}


fn main() -> Result<()> {
    set_host_os();
}