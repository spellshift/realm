use anyhow::Result;
use starlark::values::none::NoneType;

use windows_sys::Win32::System::{Memory::VirtualAlloc, Diagnostics::Debug::IMAGE_DIRECTORY_ENTRY_BASERELOC};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    System::{
        Diagnostics::Debug::{IMAGE_NT_HEADERS64,IMAGE_NT_HEADERS32,IMAGE_SECTION_HEADER},
        SystemServices::{IMAGE_DOS_HEADER},
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{VirtualAllocEx,MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE},
    },
};
use std::mem;
use std::ptr;
use std::ffi::c_void;

#[derive(Debug, Copy, Clone)]
struct BaseRelocationBlock {
    page_address: usize,
    block_size: u32,
}
#[derive(Debug, Copy, Clone)]
struct BaseRelocationEntry {
    offset: u16, // offset bits
    data_type: u16,
}

#[derive(Clone)]
struct PeFileHeaders64 {
    dos_header: IMAGE_DOS_HEADER,
    nt_headers: IMAGE_NT_HEADERS64,
    section_headers: Vec<IMAGE_SECTION_HEADER>,
}

// copy over DLL image sections to the newly allocated space for the DLL
// #[cfg(target_arch = "x86_64")]
// fn copy_dll_image_sections_to_memory_64(new_dll_base: *mut c_void, dll_bytes: Vec<u8>) -> Result<()>{
// //nt_headers: *mut IMAGE_NT_HEADERS64
//     let nt_headers = get_nt_headers_64(dll_bytes.as_ptr() as usize)?;

//     let number_of_sections = (unsafe{*nt_headers}).FileHeader.NumberOfSections;
//     let mut section_index: u16 = 0;
//     while section_index < number_of_sections {
//         let section_ref :*mut IMAGE_SECTION_HEADER = (nt_headers as usize + 264 as usize + (section_index as usize * std::mem::size_of::<IMAGE_SECTION_HEADER>() as usize) as usize) as *mut IMAGE_SECTION_HEADER;
//         println!("Section Name: {}", std::str::from_utf8(&(unsafe{*section_ref}).Name).unwrap() );
//         println!("Section VirtualAddr: {:?}", (unsafe{*section_ref}).VirtualAddress );
//         section_index = section_index + 1;
//     }
    
//     Ok(())
// }

impl PeFileHeaders64 {
    fn new(dll_bytes: Vec<u8>) -> Result<Self> {
        let dos_header_base_ref = dll_bytes.as_ptr() as usize;
        let dos_headers = unsafe { *((dos_header_base_ref) as *mut IMAGE_DOS_HEADER) };
        if dos_headers.e_magic != 23117 {
            return Err(anyhow::anyhow!("PE Magic header mismatch. File does not appear to be a PE executable."));
        }
    
        let nt_header_base_ref = dos_header_base_ref + dos_headers.e_lfanew as usize;
        let nt_headers = unsafe { *((nt_header_base_ref) as *mut IMAGE_NT_HEADERS64) };
        
        let mut section_headers_ref = unsafe{*((nt_header_base_ref + 264 as usize ) as *mut IMAGE_SECTION_HEADER)};
        let mut section_headers: Vec<IMAGE_SECTION_HEADER> = Vec::new();
        for mut section_index in 0..nt_headers.FileHeader.NumberOfSections {
            let section: IMAGE_SECTION_HEADER = section_headers_ref.clone();
            
            section_headers.push(section);
            section_index = section_index + 1;
            section_headers_ref = unsafe{
                *((
                    nt_header_base_ref + 
                    264 as usize + 
                    (section_index as usize * std::mem::size_of::<IMAGE_SECTION_HEADER>() as usize)
                ) as *mut IMAGE_SECTION_HEADER)
            };
        }    

        Ok(Self {
            dos_header: dos_headers,
            nt_headers: nt_headers,
            section_headers: section_headers,
        })
    }
}

fn get_module_handle_a(module_name: Option<String>) -> Result<isize> {
    unsafe {
        let module_handle = match module_name {
            Some(local_module_name) => {
                    GetModuleHandleA( format!("{}\0",local_module_name).as_str().as_ptr())
            },
            None => {
                    GetModuleHandleA(ptr::null())
            },
        };
        Ok(module_handle)    
    }
}

fn get_proc_address(hmodule: isize, proc_name: String) -> Result<unsafe extern "system" fn() -> isize> {
    unsafe {
        let proc_handle: unsafe extern "system" fn() -> isize = GetProcAddress(
            hmodule, 
            "LoadLibraryA\0".as_ptr()
        ).unwrap();
        Ok(proc_handle)
    }
}

fn write_vec_to_memory(dst_mem_address: *mut c_void, src_vec_bytes: Vec<u8>, max_bytes_to_write: u32) -> Result<()>{
    let mut index: u32 = 0;
    for byte in src_vec_bytes{
        unsafe { ((dst_mem_address as usize+index as usize) as *mut c_void).write_bytes(byte, 1); }
        index = index + 1;
        if index >= max_bytes_to_write {
            break;
        }
    }
    Ok(())
}

pub fn handle_dll_reflect(dll_bytes: Vec<u8>, pid: u32) -> Result<NoneType> {
    Ok(NoneType)
}

// Translated from https://www.ired.team/offensive-security/code-injection-process-injection/reflective-dll-injection
// pub fn handle_dll_reflect(dll_bytes: Vec<u8>, pid: u32) -> Result<NoneType> {
//     println!();
//     if false { println!("Ignore unused vars dll_path: {:?}, pid: {}", dll_bytes, pid); }
//     #[cfg(not(target_os = "windows"))]
//     return Err(anyhow::anyhow!("This OS isn't supported by the dll_reflect function.\nOnly windows systems are supported"));

//     // The current base address of our module.
//     // let current_process_module_base = get_module_handle_a(None)?;

//     // // Get the kernel32.dll base address
//     // let h_kernel32 = get_module_handle_a(Some("kernel32.dll".to_string()))?;

//     // let dos_headers = get_dos_headers(dll_bytes.clone().as_ptr() as usize)?;
//     #[cfg(target_arch = "x86_64")]
//     let pe_file = parse_pe_file_x64(dll_bytes);

//     Ok(NoneType)

//     #[cfg(target_arch = "x86_64")]
//     let nt_headers = get_nt_headers_64(dll_bytes.clone().as_ptr() as usize)?;
//     #[cfg(target_arch = "x86")]
//     let nt_headers = get_nt_headers_32((dll_bytes.as_ptr() as usize))?;

//     let dll_image_size = (unsafe{*nt_headers}).OptionalHeader.SizeOfImage;

//     // Allocate memory for our DLL to be loaded into
//     let new_dll_base = unsafe { VirtualAlloc(ptr::null(), dll_image_size as usize, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };

//     // Calculate the number of bytes between our images base and the newly allocated memory.
//     let image_base_delta = (new_dll_base as usize) - (unsafe{*nt_headers}).OptionalHeader.ImageBase as usize;

//     // copy over DLL image headers to the newly allocated space for the DLL
//     write_vec_to_memory(new_dll_base, dll_bytes.clone(), (unsafe{*nt_headers}).OptionalHeader.SizeOfHeaders)?;


//     // copy over DLL image sections to the newly allocated space for the DLL
//     println!("nt_headers {:?}", nt_headers as *const IMAGE_NT_HEADERS64);
//     #[cfg(target_arch = "x86_64")]
//     copy_dll_image_sections_to_memory_64(new_dll_base, dll_bytes)?;
//     #[cfg(target_arch = "x86")]
//     copy_dll_image_sections_to_memory_32(new_dll_base, dll_bytes)?;


//     // 	IMAGE_DATA_DIRECTORY relocations = ntHeaders->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC];
//     // let relocations = (unsafe{*nt_headers}).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC as usize] ;
//     // println!("relocations.Size: {:?}", relocations.Size);

//     // let relocation_table = (relocations.VirtualAddress + new_dll_base as u32) as *mut c_void;
//     // let mut relocations_processed_byte_offset: u32 = 0;

//     // while relocations_processed_byte_offset < relocations.Size {
//     //     let relocation_block = (relocation_table as u32 + relocations_processed_byte_offset) as *mut BaseRelocationBlock;
//     //     relocations_processed_byte_offset = relocations_processed_byte_offset + std::mem::size_of::<BaseRelocationBlock>() as u32;
//     //     println!("relocation_block.size: {:?}", (unsafe{*relocation_block}).block_size);
//     //     		DWORD relocationsCount = (relocationBlock->BlockSize - sizeof(BASE_RELOCATION_BLOCK)) / sizeof(BASE_RELOCATION_ENTRY);
//     //     let relocations_count: u32 = ((unsafe{*relocation_block}).block_size - std::mem::size_of::<BaseRelocationBlock>() as u32) /  std::mem::size_of::<BaseRelocationEntry>() as u32;
//     //     let relocation_entries = (relocation_table as u32 + relocations_processed_byte_offset) as *mut BaseRelocationEntry; //(relocationTable + relocationsProcessed);
//     //     println!("{:?}", relocations_count);

//     //     let mut index = 0;
//     //     while index < relocations_count {
//     //         let relocation_entry = (relocation_entries as u32 + index) as *mut BaseRelocationEntry;
//     //         println!("reloc_entry datatype: {:?}", (unsafe {*relocation_entry}).data_type);
// 	// 		if (unsafe {*relocation_entry}).data_type == 0 {
//     //             continue;
//     //         }
//     //         index = index + 1;
//     //     }

//     // }

//     Ok(NoneType)
// }

fn get_u8_vec_form_u32_vec(u32_vec: Vec<u32>) -> Result<Vec<u8>> {
    let res_u8_vec: Vec<u8> = u32_vec.iter().map(|x| if *x < u8::MAX as u32 { *x as u8 }else{ u8::MAX }).collect();
    Ok(res_u8_vec)
}

pub fn dll_reflect(dll_bytes: Vec<u32>, pid: u32) -> Result<NoneType> {
    let local_dll_bytes = get_u8_vec_form_u32_vec(dll_bytes)?;
    handle_dll_reflect(local_dll_bytes, pid)
}

#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use super::*;
    use core::time;
    use std::{process::Command, thread, path::Path, fs};
    use sysinfo::{Pid, Signal};
    use tempfile::NamedTempFile;
    use sysinfo::{ProcessExt,System,SystemExt,PidExt};

    #[test]
    fn test_dll_reflect_write_vec_to_mem() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let buf_size: usize = 16;
        let new_dll_base = unsafe { VirtualAlloc(ptr::null(), buf_size, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
        let test_buffer: Vec<u8> = vec![104, 101, 108, 108, 111, 95, 119, 111, 114, 108, 100, 95, 49, 50, 0];

        write_vec_to_memory(new_dll_base, test_buffer.clone(), buf_size as u32);

        let mut tmp_dll_base = new_dll_base.clone();
        for (index, byte) in test_buffer.iter().enumerate() {
            let res = unsafe { *(tmp_dll_base as *mut u8) };
            assert_eq!(*byte, res);
            tmp_dll_base = (tmp_dll_base as usize + 1 as usize) as *mut c_void;
        }
        Ok(())
    }

    #[test]
    fn test_dll_reflect_parse_pe_headers() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let read_in_dll_bytes = include_bytes!("..\\..\\..\\..\\tests\\create_file_dll\\target\\debug\\create_file_dll.dll");
        let dll_bytes = read_in_dll_bytes.to_vec();

        let pe_file_headers = PeFileHeaders64::new(dll_bytes)?; //get_dos_headers(dll_bytes.as_ptr() as usize)?;
        // 0x5A4D == a"ZM" == d23117 --- PE Magic number is static.
        assert_eq!(23117, pe_file_headers.dos_header.e_magic);
        // 0x020B == d523
        assert_eq!(523, pe_file_headers.nt_headers.OptionalHeader.Magic);

        let expected_section_names = vec![
            ".text\0\0\0",
            ".rdata\0\0",
            ".data\0\0\0",
            ".pdata\0\0",
            ".reloc\0\0",
        ];
        for (section_index, section) in pe_file_headers.section_headers.iter().enumerate() {
            println!("{:?}", String::from_utf8(section.Name.to_vec())?);
            assert_eq!(expected_section_names[section_index], String::from_utf8(section.Name.to_vec())?)
        }
        Ok(())
    }

    #[test]
    fn test_dll_reflect_simple() -> anyhow::Result<()>{
        const DLL_EXEC_WAIT_TIME: u64 = 5;
        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Get the path to our test dll file.
        let read_in_dll_bytes = include_bytes!("..\\..\\..\\..\\tests\\create_file_dll\\target\\debug\\create_file_dll.dll");
        let dll_bytes = read_in_dll_bytes.to_vec();

        // Out target process is notepad for stability and control.
        // The temp file is passed through an environment variable.
        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe").env("LIBTESTFILE", path.clone()).spawn();
        let target_pid = expected_process.unwrap().id();

        // Run our code.
        let _res = handle_dll_reflect(dll_bytes, target_pid)?;

        let delay = time::Duration::from_secs(DLL_EXEC_WAIT_TIME);
        thread::sleep(delay);

        // Test that the test file was created
        let test_path = Path::new(path.as_str());
        assert!(test_path.is_file());

        // Delete test file
        let _ = fs::remove_file(test_path);
        
        // kill the target process notepad
        let mut sys = System::new();
        sys.refresh_processes();
        match sys.process(Pid::from_u32(target_pid)) {
            Some(res) => {
                res.kill_with(Signal::Kill);
            },
            None => {
            },
        }

        Ok(())
    }
}

