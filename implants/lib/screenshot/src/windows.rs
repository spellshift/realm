use std::mem;
use std::ptr;
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD,
    SRCCOPY,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub fn capture_screen() -> Result<Vec<u8>, String> {
    unsafe {
        let screen_dc = GetDC(0);
        if screen_dc == 0 {
            return Err("Failed to get screen DC".to_string());
        }

        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc == 0 {
            ReleaseDC(0, screen_dc);
            return Err("Failed to create compatible DC".to_string());
        }

        let bitmap = CreateCompatibleBitmap(screen_dc, width, height);
        if bitmap == 0 {
            DeleteDC(mem_dc);
            ReleaseDC(0, screen_dc);
            return Err("Failed to create compatible bitmap".to_string());
        }

        let old_bitmap = SelectObject(mem_dc, bitmap);
        if old_bitmap == 0 {
            DeleteObject(bitmap);
            DeleteDC(mem_dc);
            ReleaseDC(0, screen_dc);
            return Err("Failed to select bitmap".to_string());
        }

        if BitBlt(mem_dc, 0, 0, width, height, screen_dc, 0, 0, SRCCOPY) == 0 {
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap);
            DeleteDC(mem_dc);
            ReleaseDC(0, screen_dc);
            return Err("Failed to BitBlt".to_string());
        }

        // Prepare BITMAPINFO
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        };

        // Calculate size
        let image_size = (width * height * 4) as usize;
        let mut pixels = vec![0u8; image_size];

        if GetDIBits(
            mem_dc,
            bitmap,
            0,
            height as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bi,
            DIB_RGB_COLORS,
        ) == 0
        {
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap);
            DeleteDC(mem_dc);
            ReleaseDC(0, screen_dc);
            return Err("Failed to GetDIBits".to_string());
        }

        // Cleanup
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(bitmap);
        DeleteDC(mem_dc);
        ReleaseDC(0, screen_dc);

        // Construct BMP
        let file_size = 14 + 40 + image_size as u32;
        let mut bmp_data = Vec::with_capacity(file_size as usize);

        // BITMAPFILEHEADER
        bmp_data.extend_from_slice(&[0x42, 0x4D]); // "BM"
        bmp_data.extend_from_slice(&file_size.to_le_bytes());
        bmp_data.extend_from_slice(&[0, 0, 0, 0]); // Reserved
        bmp_data.extend_from_slice(&[54, 0, 0, 0]); // Offset to data (14 + 40)

        // BITMAPINFOHEADER
        bmp_data.extend_from_slice(&(mem::size_of::<BITMAPINFOHEADER>() as u32).to_le_bytes());
        bmp_data.extend_from_slice(&width.to_le_bytes());
        bmp_data.extend_from_slice(&height.to_le_bytes());
        bmp_data.extend_from_slice(&(1u16).to_le_bytes()); // Planes
        bmp_data.extend_from_slice(&(32u16).to_le_bytes()); // BitCount
        bmp_data.extend_from_slice(&(0u32).to_le_bytes()); // Compression (BI_RGB)
        bmp_data.extend_from_slice(&(image_size as u32).to_le_bytes()); // SizeImage
        bmp_data.extend_from_slice(&(0i32).to_le_bytes()); // X
        bmp_data.extend_from_slice(&(0i32).to_le_bytes()); // Y
        bmp_data.extend_from_slice(&(0u32).to_le_bytes()); // ClrUsed
        bmp_data.extend_from_slice(&(0u32).to_le_bytes()); // ClrImportant

        bmp_data.extend_from_slice(&pixels);

        Ok(bmp_data)
    }
}
