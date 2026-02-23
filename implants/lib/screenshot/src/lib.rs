#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod dummy {
    pub fn capture_screen() -> Result<Vec<u8>, String> {
        Err("Unsupported platform".to_string())
    }
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
use dummy as platform;

pub fn capture_screen() -> Result<Vec<u8>, String> {
    platform::capture_screen()
}
