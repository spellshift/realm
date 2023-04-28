use anyhow::Result;
use gazebo::prelude::SliceExt;
use starlark::values::none::NoneType;

use windows_sys::Win32::System::{Memory::VirtualAlloc, Diagnostics::Debug::{IMAGE_DIRECTORY_ENTRY_BASERELOC, IMAGE_DATA_DIRECTORY, IMAGE_DIRECTORY_ENTRY_IMPORT}, SystemServices::{IMAGE_BASE_RELOCATION, IMAGE_RELOCATION, IMAGE_RELOCATION_0, IMAGE_IMPORT_DESCRIPTOR, IMAGE_ORDINAL_FLAG, IMAGE_ORDINAL_FLAG32, IMAGE_ORDINAL_FLAG64, IMAGE_IMPORT_BY_NAME}, LibraryLoader::LoadLibraryA, WindowsProgramming::{IMAGE_THUNK_DATA64, IMAGE_THUNK_DATA32}};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    System::{
        Diagnostics::Debug::{IMAGE_NT_HEADERS64,IMAGE_NT_HEADERS32,IMAGE_SECTION_HEADER},
        SystemServices::{IMAGE_DOS_HEADER},
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{VirtualAllocEx,MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE},
    },
};
use std::ffi::CStr;
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
    offset: u16,
    reloc_type: u16,
}

impl BaseRelocationEntry {
    fn new(c_bytes: u16) -> Self {
        let offset_byte_mask: u16 = 0b1111_1111_1111_0000;
        let offset = (c_bytes & offset_byte_mask) >> 4;
        // let reloc_type = c_bytes & 0xfff;
        let reloc_type_byte_mask: u16 = 0b0000_0000_0000_1111;
        let reloc_type = c_bytes & reloc_type_byte_mask;
        // let offset = c_bytes >> 12;
        Self {
            offset: offset,
            reloc_type: reloc_type,
        }
    }
    fn c_size() -> usize {
        return std::mem::size_of::<u16>();
    }
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

// Load the DLL sections (Eg: .reloc, .text, .rdata) into memory
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

// We've copied all the sections from our DLL into memory and now we need to update some of the pointers to make senes.
// On disk the memory pointers are set to the offset so inorder to update the pointer of our now in memory DLL we add the dleta between the image bases.
fn process_dll_image_relocation(new_dll_base: *mut c_void, pe_file_headers: PeFileHeaders64, image_base_delta: usize) -> Result<()>{
    let relocation_directory = pe_file_headers.nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC as usize];	
    if relocation_directory.Size == 0 {
        // No relocations to process
        return Ok(());
    }

    let mut base_image_relocation_table: *mut IMAGE_BASE_RELOCATION = (new_dll_base as usize + relocation_directory.VirtualAddress as usize) as *mut IMAGE_BASE_RELOCATION;
    loop {
        let relocation_block = (unsafe{*base_image_relocation_table as IMAGE_BASE_RELOCATION});
        let relocation_block_size = relocation_block.SizeOfBlock;
        if relocation_block_size == 0 {
            break;
        }
        println!("relocation_block.VirtualAddress {:#04x}", relocation_block.VirtualAddress);
        println!("relocation_block.SizeOfBlock {:#04x}", relocation_block.SizeOfBlock);
        // This needs to be calculated since the relocation_block doesn't track it.
        // Luckily the relocation_entry is a static size: u16.
        // Unfortunately the struct uses offset bits which is annoying in Rust.
        // c++ struct:
        // typedef struct BASE_RELOCATION_ENTRY {
        //      USHORT Offset : 12;
        //      USHORT Type : 4;
        // } BASE_RELOCATION_ENTRY, *PBASE_RELOCATION_ENTRY;
        let relocation_block_entries_count = (relocation_block.SizeOfBlock as usize - std::mem::size_of::<IMAGE_BASE_RELOCATION>() as usize) / BaseRelocationEntry::c_size();
        println!("relocation_block_entries_count {:#04x}", relocation_block_entries_count);

        let mut relocation_entry_ptr: *mut u16 = (base_image_relocation_table as usize + std::mem::size_of::<IMAGE_BASE_RELOCATION>() as usize) as *mut u16;
        for _index in 1..relocation_block_entries_count {
            let relocation_entry = BaseRelocationEntry::new(unsafe{*relocation_entry_ptr});

            // If the reloction type isn't 0 try relocating it.
            if relocation_entry.reloc_type != 0 {
                // Calculate the adress of the relocation record in the new_dll memory.
                let ptr_to_relocation_to_update = (new_dll_base as usize + relocation_block.VirtualAddress as usize + relocation_entry.offset as usize) as *mut u32;
                // Calculate the relocation value relocation record + the image delta.
                let new_reloc_value = unsafe { *ptr_to_relocation_to_update } as u32 + image_base_delta as u32;
                // Update the value we point at.
                unsafe { *ptr_to_relocation_to_update = new_reloc_value };
                // Update the relocation entry pointer by incrementing by the size of an entry.
                relocation_entry_ptr = (relocation_entry_ptr as usize + BaseRelocationEntry::c_size()) as *mut u16;    
            }
        }

        base_image_relocation_table = (base_image_relocation_table as usize + relocation_block_size as usize) as *mut IMAGE_BASE_RELOCATION;
    }
    // uiValueB = (ULONG_PTR)&((PIMAGE_NT_HEADERS)uiHeaderValue)->OptionalHeader.DataDirectory[ IMAGE_DIRECTORY_ENTRY_BASERELOC ];
    Ok(())
}

//def IMAGE_SNAP_BY_ORDINAL(Ordinal): return ((Ordinal & IMAGE_ORDINAL_FLAG) != 0)
fn image_snap_by_ordinal(ordinal: u64) -> bool{
    #[cfg(target_arch = "x86_64")]
    return (ordinal & IMAGE_ORDINAL_FLAG64) != 0;
    #[cfg(target_arch = "x86")]   
    return (ordinal & IMAGE_ORDINAL_FLAG32) != 0;
}

// def IMAGE_ORDINAL(Ordinal): return (Ordinal & 65535)
fn image_ordinal(ordinal: u64) -> u64 {
    return ordinal & 65535;
}

fn update_library_first_thunk_ref(mut library_first_thunk_ref: *mut IMAGE_THUNK_DATA64 ) -> *mut IMAGE_THUNK_DATA64 {
    library_first_thunk_ref = (library_first_thunk_ref as usize + 1 as usize) as *mut IMAGE_THUNK_DATA64;
    return library_first_thunk_ref;
}

fn process_import_address_tables(new_dll_base: *mut c_void, pe_file_headers: PeFileHeaders64, image_base_delta: usize) -> Result<()>{
    let import_directory = pe_file_headers.nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];
    let import_directory_block_size = import_directory.Size;
	
    if import_directory.Size == 0 {
        // No relocations to process
        return Ok(());
    }

    let mut base_image_import_table: *mut IMAGE_IMPORT_DESCRIPTOR = (new_dll_base as usize + import_directory.VirtualAddress as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    loop {
        let import_table_entry = unsafe{*base_image_import_table};
        if import_table_entry.Name == 0 {
            break;
        }
        println!("NameRVA: {:#06x}", import_table_entry.Name);

        let slice = (new_dll_base as usize + import_table_entry.Name as usize) as *const i8;
        let library_name = (unsafe { CStr::from_ptr(slice) }).to_str()?;
        println!("library_name: {}", library_name); // gotta cut the null terminated strings out.
        let library_handle = unsafe { LoadLibraryA( format!("{}\0",library_name).as_str().as_ptr()) };
        if library_handle != 0 {
            #[cfg(target_arch = "x86_64")]
            let mut library_first_thunk_ref = (new_dll_base as usize + import_table_entry.FirstThunk as usize) as *mut IMAGE_THUNK_DATA64;
            #[cfg(target_arch = "x86")]
            let mut library_first_thunk_ref = (new_dll_base as usize + import_table_entry.FirstThunk as usize) as *mut IMAGE_THUNK_DATA32;

            loop {
                let mut library_first_thunk = unsafe{(*library_first_thunk_ref)};

                println!("{:?}", library_first_thunk_ref);
                // Access of a union field is unsafe
                if unsafe{library_first_thunk.u1.AddressOfData} == 0 {
                    break;
                }
                if image_snap_by_ordinal(unsafe{library_first_thunk.u1.Ordinal}) {
                    println!("HERE1");
                    // LPCSTR functionOrdinal = (LPCSTR)IMAGE_ORDINAL(thunk->u1.Ordinal);
                    let function_ordinal = image_ordinal(unsafe{library_first_thunk.u1.Ordinal}) as u8;
                    // thunk->u1.Function = (DWORD_PTR)GetProcAddress(library, functionOrdinal);
                    println!("HERE2");
                    library_first_thunk.u1.Function = match unsafe { GetProcAddress(library_handle, &function_ordinal) } {
                        Some(local_thunk_function) => {
                            if cfg!(target_pointer_width = "64") {
                                local_thunk_function as u64
                            } else if cfg!(target_pointer_width = "32") {
                                (local_thunk_function as u32).into()
                            } else {
                                return Err(anyhow::anyhow!("Target pointer width isnt 64 or 32."));
                            }
                        },
                        None => unsafe{library_first_thunk.u1.Function},
                    };
                    println!("HERE3");
                    println!("library_first_thunk.u1.Function: {:?}", unsafe{library_first_thunk.u1.Function})
                } else {
                    println!("HERE4");
                    // PIMAGE_IMPORT_BY_NAME functionName = (PIMAGE_IMPORT_BY_NAME)((DWORD_PTR)dllBase + thunk->u1.AddressOfData);
                    let function_name_ref: *mut IMAGE_IMPORT_BY_NAME = (new_dll_base as usize + unsafe{library_first_thunk.u1.AddressOfData} as usize) as *mut IMAGE_IMPORT_BY_NAME;
                    println!("function_name: {:?}", (unsafe { CStr::from_ptr( (*function_name_ref).Name.as_ptr() as *const i8) }).to_str()?);
                    let function_name = unsafe{*function_name_ref}.Name[0];
                    // DWORD_PTR functionAddress = (DWORD_PTR)GetProcAddress(library, functionName->Name);
                    // thunk->u1.Function = functionAddress;
                    println!("HERE5");

                    library_first_thunk.u1.Function = match unsafe { GetProcAddress(library_handle, &function_name) } {
                        Some(local_thunk_function) => {
                            if cfg!(target_pointer_width = "64") {
                                local_thunk_function as u64
                            } else if cfg!(target_pointer_width = "32") {
                                (local_thunk_function as u32).into()
                            } else {
                                return Err(anyhow::anyhow!("Target pointer width isnt 64 or 32."));
                            }
                        },
                        None => unsafe{library_first_thunk.u1.Function},
                    };
                    println!("HERE6");
                    println!("library_first_thunk.u1.Function: {:?}", unsafe{library_first_thunk.u1.Function})
                }
                println!("HERE7");
                // Original thunk ref and new thunk ref need to be updated.
                library_first_thunk_ref = update_library_first_thunk_ref(library_first_thunk_ref);
                
            }
        }
        println!("HERE7");
        base_image_import_table = (base_image_import_table as usize + std::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>() as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    }

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
    process_dll_image_relocation(new_dll_base, pe_header.clone(), image_base_delta)?;

	// resolve import address table
    process_import_address_tables(new_dll_base, pe_header.clone(), image_base_delta)?;

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
    fn test_dll_reflect_new_BaseRelocationEntry() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let test_entry: u16 = 0xA008;
        let base_reloc_entry = BaseRelocationEntry::new(test_entry);
        assert_eq!(base_reloc_entry.offset, 2560);
        assert_eq!(base_reloc_entry.reloc_type, 8);
        Ok(())
    }

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

