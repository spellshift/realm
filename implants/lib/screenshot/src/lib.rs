#[cfg(feature = "stdlib")]
use image::RgbaImage;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(feature = "stdlib")]
pub fn capture_monitors() -> Result<Vec<RgbaImage>, String> {
    #[cfg(target_os = "linux")]
    {
        linux::capture_monitors()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let monitors = xcap::Monitor::all().map_err(|e| e.to_string())?;
        let mut images = Vec::new();
        for monitor in monitors {
            let image = monitor.capture_image().map_err(|e| e.to_string())?;
            images.push(image);
        }
        Ok(images)
    }
}

#[cfg(not(feature = "stdlib"))]
pub fn capture_monitors() -> Result<Vec<()>, String> {
    Err("capture_monitors requires the stdlib feature".into())
}
