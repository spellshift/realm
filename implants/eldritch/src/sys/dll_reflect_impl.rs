use anyhow::Result;
use starlark::values::none::NoneType;

use windows_sys::Win32::System::{Memory::VirtualAlloc, Diagnostics::Debug::{IMAGE_DIRECTORY_ENTRY_BASERELOC, IMAGE_DATA_DIRECTORY}, SystemServices::{IMAGE_BASE_RELOCATION, IMAGE_RELOCATION, IMAGE_RELOCATION_0}};
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
    section_headers: Vec<IMAGE_SECTION_HEADER>
}

struct RelocTable {
    page_rva: u32,
    block_size: u32,
    relocation_entries: Vec<u16>,
}

// Pares the PE file from a series of bytes
#[cfg(target_arch = "x86_64")]
impl PeFileHeaders64 {
    fn new(dll_bytes: Vec<u8>) -> Result<Self> {
        // DOS Headers
        let dos_header_base_ref = dll_bytes.as_ptr() as usize;
        let dos_headers = unsafe { *((dos_header_base_ref) as *mut IMAGE_DOS_HEADER) };
        if dos_headers.e_magic != 23117 {
            return Err(anyhow::anyhow!("PE Magic header mismatch. File does not appear to be a PE executable."));
        }
    
        // NT Headers
        let nt_header_base_ref = dos_header_base_ref + dos_headers.e_lfanew as usize;
        let nt_headers = unsafe { *((nt_header_base_ref) as *mut IMAGE_NT_HEADERS64) };
        
        // Section Headers
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

        if section_headers.len() != nt_headers.FileHeader.NumberOfSections as usize {
            return Err(anyhow::anyhow!(format!("PE section count {} doesn't match nt_header.FileHeader.NumberOfSections {}", section_headers.len(), nt_headers.FileHeader.NumberOfSections)));
        }

        // Section Reloc entries
        let mut reloc_table: Vec<RelocTable> = Vec::new();
        let mut reloc_section_ptr: *mut c_void = 0 as *mut c_void;
        for section_header in section_headers {
            if String::from_utf8(section_header.Name.to_vec())?.contains(".reloc") {
                reloc_section_ptr = (dos_header_base_ref as usize + section_header.PointerToRelocations as usize) as *mut c_void;
            }
        }
        if !reloc_section_ptr.is_null() {
            loop {
                let mut tmp_base_reloc = unsafe{*(reloc_section_ptr as *mut IMAGE_BASE_RELOCATION)};
                if tmp_base_reloc.SizeOfBlock == 0 {
                    break;
                }
                let reloc_table = 
    
            }
        }


        Ok(Self {
            dos_header: dos_headers,
            nt_headers: nt_headers,
            section_headers: section_headers,
        })
    }
}

#[cfg(target_arch = "x86")]
impl PeFileHeaders32 {
    fn new(dll_bytes: Vec<u8>) -> Result<Self> {
        let dos_header_base_ref = dll_bytes.as_ptr() as usize;
        let dos_headers = unsafe { *((dos_header_base_ref) as *mut IMAGE_DOS_HEADER) };
        if dos_headers.e_magic != 23117 {
            return Err(anyhow::anyhow!("PE Magic header mismatch. File does not appear to be a PE executable."));
        }
    
        let nt_header_base_ref = dos_header_base_ref + dos_headers.e_lfanew as usize;
        let nt_headers = unsafe { *((nt_header_base_ref) as *mut IMAGE_NT_HEADERS32) };
        
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

        if section_headers.len() != nt_headers.FileHeader.NumberOfSections as usize {
            return Err(anyhow::anyhow!(format!("PE section count {} doesn't match nt_header.FileHeader.NumberOfSections {}", section_headers.len(), nt_headers.FileHeader.NumberOfSections)));
        }

        Ok(Self {
            dos_header: dos_headers,
            nt_headers: nt_headers,
            section_headers: section_headers,
        })
    }
}

fn get_module_handle_a(module_name: Option<String>) -> Result<usize> {
    unsafe {
        let module_handle = match module_name {
            Some(local_module_name) => {
                    GetModuleHandleA( format!("{}\0",local_module_name).as_str().as_ptr())
            },
            None => {
                    GetModuleHandleA(ptr::null())
            },
        };
        Ok(module_handle as usize)
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

pub fn write_n_bytes_to_memory(dst_mem_address: *mut c_void, src_mem_address: *const c_void, max_bytes_to_write: u32) -> Result<()> {
    let mut index: u32 = 0;
    while index <  max_bytes_to_write {
        let tmp_byte_val = unsafe { *((src_mem_address as usize + index as usize) as *const u8) };
        unsafe { ((dst_mem_address as usize+index as usize) as *mut c_void).write_bytes(tmp_byte_val, 1) };
        index = index + 1;
    }
    Ok(())
}

fn relocate_dll_image_sections(new_dll_base: *mut c_void, old_dll_bytes: *const c_void, pe_file_headers: PeFileHeaders64) -> Result<()> {

    for (_section_index, section) in pe_file_headers.section_headers.iter().enumerate() {
        // LPVOID sectionDestination = (LPVOID)((DWORD_PTR)dllBase + (DWORD_PTR)section->VirtualAddress);
        let section_destination = new_dll_base as usize + section.VirtualAddress as usize;
        // LPVOID sectionBytes = (LPVOID)((DWORD_PTR)dllBytes + (DWORD_PTR)section->PointerToRawData);
        let section_bytes = old_dll_bytes as usize + section.PointerToRawData as usize;
        // std::memcpy(sectionDestination, sectionBytes, section->SizeOfRawData);
        write_n_bytes_to_memory(section_destination as *mut c_void, section_bytes as *const c_void, section.SizeOfRawData)?;

        println!("{:?}", String::from_utf8(section.Name.to_vec())?);
    }

    Ok(())
}

fn process_dll_image_relocation(new_dll_base: *mut c_void, old_dll_bytes: *const c_void, pe_file_headers: PeFileHeaders64, image_base_delta: usize) -> Result<()>{
    // let relocation_directory = pe_file_headers.nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC as usize];	
    // if relocation_directory.Size == 0 {
    //     // No relocations to process
    //     return Ok(());
    // }

    // let mut ptr_to_reloc_table: *mut c_void;// = old_dll_bytes as usize + pe_file_headers.nt_headers.OptionalHeader.DataDirectory
    // for section in pe_file_headers.section_headers {
    //     if String::from_utf8(section.Name.to_vec())?.contains(".reloc") {
    //         ptr_to_reloc_table = (old_dll_bytes as usize + section.PointerToRawData as usize) as *mut IMAGE_RELOCATION_0
    //         println!(".reloc section.PointerToRawData: {:?}", section.PointerToRawData);
    //     }
    // }

    // let mut base_image_relocation_table = (new_dll_base as usize + relocation_directory.VirtualAddress as usize) as *mut IMAGE_BASE_RELOCATION;
    // loop {
    //     let relocation_block = (unsafe{*base_image_relocation_table as IMAGE_BASE_RELOCATION});
    //     let relocation_block_size = relocation_block.SizeOfBlock;
    //     if relocation_block_size == 0 {
    //         break;
    //     }

    //     let destination_virtual_address = old_dll_bytes as usize + relocation_block.VirtualAddress as usize;
    //     let relocation_block_entries_count = relocation_block.SizeOfBlock - std::mem::size_of::<IMAGE_BASE_RELOCATION>() as u32 / std::mem::size_of::<IMAGE_RELOCATION>() as u32;
    //     println!("relocation_block_entries_count: {:?}", relocation_block_entries_count);
    //     // Getting the right number of entries 6 but the entries count is way off but not changing.

    //     base_image_relocation_table = (base_image_relocation_table as usize + relocation_block_size as usize) as *mut IMAGE_BASE_RELOCATION;
    // }
    // uiValueB = (ULONG_PTR)&((PIMAGE_NT_HEADERS)uiHeaderValue)->OptionalHeader.DataDirectory[ IMAGE_DIRECTORY_ENTRY_BASERELOC ];
    Ok(())
}
pub fn handle_dll_reflect(dll_bytes: Vec<u8>, pid: Option<u32>) -> Result<NoneType> {
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!("This OS isn't supported by the dll_reflect function.\nOnly windows systems are supported"));

    #[cfg(target_arch = "x86_64")]
    let pe_header = PeFileHeaders64::new(dll_bytes.clone())?;
    #[cfg(target_arch = "x86")]
    let pe_header = PeFileHeaders32::new(dll_bytes.clone())?;

    // Allocate memory for our DLL to be loaded into
    let new_dll_base = unsafe { VirtualAlloc(ptr::null(), pe_header.nt_headers.OptionalHeader.SizeOfImage as usize, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
    
    // Write our DLL headers into the newly allocated memory.
    write_vec_to_memory(new_dll_base, dll_bytes.clone(), pe_header.nt_headers.OptionalHeader.SizeOfHeaders)?;

    // copy over DLL image sections to the newly allocated space for the DLL
    relocate_dll_image_sections(new_dll_base, dll_bytes.clone().as_ptr() as *const c_void, pe_header.clone())?;

    // Get distance between new dll memory and on disk image base.
    if pe_header.nt_headers.OptionalHeader.ImageBase as usize > new_dll_base as usize {
        return Err(anyhow::anyhow!("image_base ptr was greater than dll_mem ptr."));
    }
    let image_base_delta = new_dll_base as usize - pe_header.nt_headers.OptionalHeader.ImageBase as usize;

    // perform image base relocations
    process_dll_image_relocation(new_dll_base, dll_bytes.clone().as_ptr() as *const c_void, pe_header.clone(), image_base_delta)?;

    // get this module's image base address
    let current_process_module_base = get_module_handle_a(None)?;
    Ok(NoneType)
}



fn get_u8_vec_form_u32_vec(u32_vec: Vec<u32>) -> Result<Vec<u8>> {
    let res_u8_vec: Vec<u8> = u32_vec.iter().map(|x| if *x < u8::MAX as u32 { *x as u8 }else{ u8::MAX }).collect();
    Ok(res_u8_vec)
}

pub fn dll_reflect(dll_bytes: Vec<u32>, pid: u32) -> Result<NoneType> {
    let local_dll_bytes = get_u8_vec_form_u32_vec(dll_bytes)?;
    handle_dll_reflect(local_dll_bytes, Some(pid))
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
    fn test_dll_reflect_write_n_bytes_to_memory() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let buf_size: usize = 16;
        let new_dll_base = unsafe { VirtualAlloc(ptr::null(), buf_size, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
        let test_buffer: Vec<u8> = vec![104, 101, 108, 108, 111, 95, 119, 111, 114, 108, 100, 95, 49, 50, 0];
        let test_buffer_ref = test_buffer.as_ptr() as *const c_void;

        write_n_bytes_to_memory(new_dll_base, test_buffer_ref, buf_size as u32);

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
        let _res = handle_dll_reflect(dll_bytes, Some(target_pid))?;

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

