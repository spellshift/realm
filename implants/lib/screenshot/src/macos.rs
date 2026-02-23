use std::os::raw::c_void;
use std::slice;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGMainDisplayID() -> u32;
    fn CGDisplayCreateImage(displayID: u32) -> *mut c_void;
    fn CGImageGetDataProvider(image: *mut c_void) -> *mut c_void;
    fn CGImageGetWidth(image: *mut c_void) -> usize;
    fn CGImageGetHeight(image: *mut c_void) -> usize;
    fn CGImageRelease(image: *mut c_void);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CGDataProviderCopyData(provider: *mut c_void) -> *mut c_void;
    fn CFDataGetBytePtr(data: *mut c_void) -> *const u8;
    fn CFDataGetLength(data: *mut c_void) -> isize;
    fn CFRelease(cf: *mut c_void);
}

pub fn capture_screen() -> Result<Vec<u8>, String> {
    unsafe {
        let display_id = CGMainDisplayID();
        let image = CGDisplayCreateImage(display_id);
        if image.is_null() {
            return Err("Failed to create image".to_string());
        }

        let provider = CGImageGetDataProvider(image);
        if provider.is_null() {
            CGImageRelease(image);
            return Err("Failed to get provider".to_string());
        }

        let data = CGDataProviderCopyData(provider);
        if data.is_null() {
            CGImageRelease(image);
            return Err("Failed to copy data".to_string());
        }

        let width = CGImageGetWidth(image);
        let height = CGImageGetHeight(image);

        let ptr = CFDataGetBytePtr(data);
        let len = CFDataGetLength(data);

        let pixels = slice::from_raw_parts(ptr, len as usize).to_vec();

        CFRelease(data);
        CGImageRelease(image);

        // Construct BMP
        let image_size = pixels.len() as u32;
        let file_size = 14 + 40 + image_size;
        let mut bmp_data = Vec::with_capacity(file_size as usize);

        // BITMAPFILEHEADER
        bmp_data.extend_from_slice(&[0x42, 0x4D]); // "BM"
        bmp_data.extend_from_slice(&file_size.to_le_bytes());
        bmp_data.extend_from_slice(&[0, 0, 0, 0]);
        bmp_data.extend_from_slice(&[54, 0, 0, 0]);

        // BITMAPINFOHEADER
        bmp_data.extend_from_slice(&(40u32).to_le_bytes());
        bmp_data.extend_from_slice(&(width as i32).to_le_bytes());
        bmp_data.extend_from_slice(&(-(height as i32)).to_le_bytes()); // Negative height for top-down
        bmp_data.extend_from_slice(&(1u16).to_le_bytes());
        bmp_data.extend_from_slice(&(32u16).to_le_bytes());
        bmp_data.extend_from_slice(&(0u32).to_le_bytes()); // BI_RGB
        bmp_data.extend_from_slice(&(image_size).to_le_bytes());
        bmp_data.extend_from_slice(&(0i32).to_le_bytes());
        bmp_data.extend_from_slice(&(0i32).to_le_bytes());
        bmp_data.extend_from_slice(&(0u32).to_le_bytes());
        bmp_data.extend_from_slice(&(0u32).to_le_bytes());

        bmp_data.extend_from_slice(&pixels);

        Ok(bmp_data)
    }
}
