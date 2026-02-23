use libc::{c_char, c_int, c_ulong, c_void, dlopen, dlsym, RTLD_LAZY};
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::slice;

#[repr(C)]
struct XImage {
    width: c_int,
    height: c_int,
    xoffset: c_int,
    format: c_int,
    data: *mut c_char,
    byte_order: c_int,
    bitmap_unit: c_int,
    bitmap_bit_order: c_int,
    bitmap_pad: c_int,
    depth: c_int,
    bytes_per_line: c_int,
    bits_per_pixel: c_int,
    red_mask: c_ulong,
    green_mask: c_ulong,
    blue_mask: c_ulong,
    obdata: *mut c_void,
    funcs: XImageFuncs,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct XImageFuncs {
    create_image: *mut c_void,
    destroy_image: *mut c_void,
    get_pixel: *mut c_void,
    put_pixel: *mut c_void,
    sub_image: *mut c_void,
    add_pixel: *mut c_void,
}

pub fn capture_screen() -> Result<Vec<u8>, String> {
    unsafe {
        let lib_name = CString::new("libX11.so").unwrap();
        let mut handle = dlopen(lib_name.as_ptr(), RTLD_LAZY);
        if handle.is_null() {
            let lib_name6 = CString::new("libX11.so.6").unwrap();
            handle = dlopen(lib_name6.as_ptr(), RTLD_LAZY);
            if handle.is_null() {
                return Err("Failed to load libX11".to_string());
            }
        }

        let sym = |name: &str| {
            let s = CString::new(name).unwrap();
            dlsym(handle, s.as_ptr())
        };

        let x_open_display: unsafe extern "C" fn(*const c_char) -> *mut c_void =
            mem::transmute(sym("XOpenDisplay"));
        let x_close_display: unsafe extern "C" fn(*mut c_void) -> c_int =
            mem::transmute(sym("XCloseDisplay"));
        let x_default_root_window: unsafe extern "C" fn(*mut c_void) -> c_ulong =
            mem::transmute(sym("XDefaultRootWindow"));
        let x_get_image: unsafe extern "C" fn(
            *mut c_void,
            c_ulong,
            c_int,
            c_int,
            c_int,
            c_int,
            c_ulong,
            c_int,
        ) -> *mut XImage = mem::transmute(sym("XGetImage"));
        let x_display_width: unsafe extern "C" fn(*mut c_void, c_int) -> c_int =
            mem::transmute(sym("XDisplayWidth"));
        let x_display_height: unsafe extern "C" fn(*mut c_void, c_int) -> c_int =
            mem::transmute(sym("XDisplayHeight"));
        let x_default_screen: unsafe extern "C" fn(*mut c_void) -> c_int =
            mem::transmute(sym("XDefaultScreen"));
        let x_destroy_image: unsafe extern "C" fn(*mut XImage) -> c_int =
             mem::transmute(sym("XDestroyImage"));


        if x_open_display as usize == 0 {
            return Err("Missing symbols".to_string());
        }

        let display = x_open_display(ptr::null());
        if display.is_null() {
            return Err("Failed to open display".to_string());
        }

        let root = x_default_root_window(display);
        let screen = x_default_screen(display);
        let width = x_display_width(display, screen);
        let height = x_display_height(display, screen);

        // ZPixmap = 2, AllPlanes = !0
        let image = x_get_image(
            display,
            root,
            0,
            0,
            width as c_int,
            height as c_int,
            !0,
            2,
        );
        if image.is_null() {
            x_close_display(display);
            return Err("Failed to get image".to_string());
        }

        let img = &*image;
        if img.bits_per_pixel != 32 {
            // Need to implement other depths or fail
             x_destroy_image(image);
             x_close_display(display);
             return Err(format!("Unsupported depth: {}", img.bits_per_pixel));
        }

        let len = (img.bytes_per_line * img.height) as usize;
        let data_slice = slice::from_raw_parts(img.data as *const u8, len);
        let pixels = data_slice.to_vec();

        x_destroy_image(image);
        x_close_display(display);

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
        bmp_data.extend_from_slice(&(-height as i32).to_le_bytes()); // Negative height for top-down
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
