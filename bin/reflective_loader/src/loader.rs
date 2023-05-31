use anyhow::Result;
use windows_sys::Win32::{System::{Memory::VirtualAlloc, Diagnostics::Debug::{IMAGE_DIRECTORY_ENTRY_BASERELOC, IMAGE_DATA_DIRECTORY, IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_SECTION_HEADER_0}, SystemServices::{IMAGE_BASE_RELOCATION, IMAGE_IMPORT_DESCRIPTOR, IMAGE_ORDINAL_FLAG64, IMAGE_IMPORT_BY_NAME, IMAGE_REL_BASED_DIR64, IMAGE_REL_BASED_HIGHLOW, DLL_PROCESS_ATTACH}, LibraryLoader::LoadLibraryA, WindowsProgramming::IMAGE_THUNK_DATA64}, Foundation::{HINSTANCE, BOOL}};
use windows_sys::Win32::{
    System::{
        Diagnostics::Debug::{IMAGE_NT_HEADERS64,IMAGE_SECTION_HEADER},
        SystemServices::{IMAGE_DOS_HEADER},
        LibraryLoader::{GetProcAddress},
        Memory::{MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE},
    },
};
use core::ffi::CStr;
use core::ptr;
use core::ffi::c_void;

#[allow(non_camel_case_types)]
type fnDllMain =
    unsafe extern "system" fn(module: HINSTANCE, call_reason: u32, reserved: *mut c_void) -> BOOL;

#[derive(Debug, Copy, Clone)]
struct BaseRelocationEntry {
    offset: u16,
    reloc_type: u16,
}

impl BaseRelocationEntry {
    fn new(c_bytes: u16) -> Self {
        let reloc_type_bit_mask: u16 = 0b1111_0000_0000_0000;
        let reloc_type = (c_bytes & reloc_type_bit_mask) >> 12;
        let offset_bit_mask: u16 = 0b0000_1111_1111_1111;
        let offset = c_bytes & offset_bit_mask;
        Self {
            offset: offset,
            reloc_type: reloc_type,
        }
    }
    fn c_size() -> usize {
        return core::mem::size_of::<u16>();
    }
}

struct PeFileHeaders64 {
    dos_header: IMAGE_DOS_HEADER,
    nt_headers: IMAGE_NT_HEADERS64,
    section_headers: [IMAGE_SECTION_HEADER; 25], // Assuming 25 - this is hopefully the most sections that a PE file can have?
}


// Pares the PE file from a series of bytes
#[cfg(target_arch = "x86_64")]
impl PeFileHeaders64 {
    fn new(dll_bytes: *mut c_void) -> Result<Self> {
        // DOS Headers
        let dos_header_base_ref = dll_bytes as usize;
        let dos_headers = unsafe { *((dos_header_base_ref) as *mut IMAGE_DOS_HEADER) };
        if dos_headers.e_magic != 0x5A4D {
            return Err(anyhow::anyhow!("PE Magic header mismatch. Expected 0x5A4D == MZ == 21117. File does not appear to be a PE executable."));
        }

        // NT Headers
        let nt_header_base_ref = dos_header_base_ref + dos_headers.e_lfanew as usize;
        let nt_headers = unsafe { *((nt_header_base_ref) as *mut IMAGE_NT_HEADERS64) };

        if nt_headers.Signature != 0x4550 {
            return Err(anyhow::anyhow!("NT Signature mismatch. Expected 0x4550 == PE == 17744. File does not appear to be a PE executable."))
        }

        // Section Headers
        let null_section = IMAGE_SECTION_HEADER{ 
            Name: [0; 8], 
            Misc: IMAGE_SECTION_HEADER_0 { 
                PhysicalAddress: 0, 
            },
            VirtualAddress: 0, 
            SizeOfRawData: 0, 
            PointerToRawData: 0, 
            PointerToRelocations: 0, 
            PointerToLinenumbers: 0, 
            NumberOfRelocations: 0, 
            NumberOfLinenumbers: 0, 
            Characteristics: 0
        };
        let mut section_headers: [IMAGE_SECTION_HEADER; 25] = [null_section; 25];
        let valid_section_headers = 
            [".rdata", ".data",".text",".pdata",".reloc",".bss",".cormeta",".debug$F",".debug$P","debug$S",
            ".debug$T",".drective",".edata",".idata",".pdata",".idlsym",".rsrc",".sbss",".sdata",".srdata",
            ".sxdata",".tls",".tls$",".vsdata",".xdata"];

        let mut cur_section_ref = (nt_header_base_ref + 264 as usize ) as *mut IMAGE_SECTION_HEADER;
        for section_index in 0..nt_headers.FileHeader.NumberOfSections {
            let cur_section = unsafe { *cur_section_ref.clone() };
            
            let section_name_tmp_ref = unsafe{core::slice::from_raw_parts(cur_section.Name.as_ptr(), 8)};
            let section_name_tmp = core::str::from_utf8(section_name_tmp_ref)?;

            if valid_section_headers.contains( &section_name_tmp ) {
                return Err(anyhow::anyhow!("Section header name {} unknown. PE file paresing failed.", section_name_tmp ));
            }
            section_headers[section_index as usize] = cur_section;

            cur_section_ref =
                    (cur_section_ref as usize + core::mem::size_of::<IMAGE_SECTION_HEADER>() as usize) as *mut IMAGE_SECTION_HEADER 
        }
        // if section_headers.len() != nt_headers.FileHeader.NumberOfSections as usize {
        //     return Err(anyhow::anyhow!("PE section count {} doesn't match nt_header.FileHeader.NumberOfSections {}", section_headers.len(), nt_headers.FileHeader.NumberOfSections));
        // }

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
        unimplemented!("x86 isn't supported yet")
    }
}

// Load the DLL sections (Eg: .reloc, .text, .rdata) into memory
fn relocate_dll_image_sections(new_dll_base: *mut c_void, old_dll_bytes: *const c_void, pe_file_headers: &PeFileHeaders64) -> Result<()> {
    for (section_index, section) in pe_file_headers.section_headers.iter().enumerate() {
        if section_index >= pe_file_headers.nt_headers.FileHeader.NumberOfSections as usize { break; }
        // LPVOID sectionDestination = (LPVOID)((DWORD_PTR)dllBase + (DWORD_PTR)section->VirtualAddress);
        let section_destination = new_dll_base as usize + section.VirtualAddress as usize;
        // LPVOID sectionBytes = (LPVOID)((DWORD_PTR)dllBytes + (DWORD_PTR)section->PointerToRawData);
        let section_bytes = old_dll_bytes as usize + section.PointerToRawData as usize;
        // std::memcpy(sectionDestination, sectionBytes, section->SizeOfRawData);
        unsafe{core::ptr::copy(section_bytes as *const c_void, section_destination as *mut c_void, section.SizeOfRawData as usize)}
    }

    Ok(())
}

// The relocation table in `.reloc` is used to help load a PE file when it's base address 
// does not match the expected address (which is common). The expected base address is 
// stored in Nt Header ---> Optional Header ---> `ImageBase`. This is the address that all 
// pointers in the code have been hardcoded to work with. To update these hardcoded values 
// we'll rebase the loaded image. To rebase the loaded image the loader will read through 
// `.reloc` looping over the relocation blocks (`IMAGE_BASE_RELOCATION`). Blocks loosely 
// correlate to PE sections Eg. `.text`. Each block has a number of 2 byte entries 
// (offset: 12bits, type: 4bits). Each entry corresponds to a hardcoded pointer in memory 
// that will need to be updated. The loader will loop over each entry in the block using 
// the offset to determine where in the loaded section a reference needs to be updated. 
// The address of the hardcoded reference can be calculated as: 
// (new_dll_base as usize + relocation_block.VirtualAddress as usize + relocation_entry.offset as usize) as *mut usize;
// The hardcoded reference is then updated by adding the image base delta. The difference 
// between the hardcoded image base `NtHeader.OptionalHeader.ImageBase` and the image base 
// of the newly loaded PE.
// https://0xrick.github.io/win-internals/pe7/
// http://research32.blogspot.com/2015/01/base-relocation-table.html
fn process_dll_image_relocation(new_dll_base: *mut c_void, pe_file_headers: &PeFileHeaders64, image_base_delta: usize) -> Result<()>{
    let relocation_directory: IMAGE_DATA_DIRECTORY = pe_file_headers.nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC as usize];	
    if relocation_directory.Size == 0 {
        // No relocations to process
        return Ok(());
    }

    let mut relocation_block_ref: *mut IMAGE_BASE_RELOCATION = 
        (new_dll_base as usize + relocation_directory.VirtualAddress as usize) as *mut IMAGE_BASE_RELOCATION;
    // println!("image_base_delta:     {}", image_base_delta);
    // println!("relocation_block_ref: {:#04x}", (relocation_block_ref as usize));
    // 	while (relocationsProcessed < relocations.Size) 
    loop {
        // if relocation_block_ref as usize > (new_dll_base as usize + relocation_directory.Size as usize) {
        //     println!("Stopping a run away train");
        //     break;
        // }
        let relocation_block = unsafe{*relocation_block_ref as IMAGE_BASE_RELOCATION};
        if relocation_block.SizeOfBlock == 0 ||
            relocation_block.VirtualAddress == 0 {
            break;
        }

        // println!("relocation_block.VirtualAddress {:#04x}", relocation_block.VirtualAddress);
        // println!("relocation_block.SizeOfBlock {:#04x}", relocation_block.SizeOfBlock);
        // This needs to be calculated since the relocation_block doesn't track it.
        // Luckily the relocation_entry is a static size: u16.
        // Unfortunately the struct uses offset bits which is annoying in Rust.
        // c++ struct:
        // typedef struct BASE_RELOCATION_ENTRY {
        //      USHORT Offset : 12;
        //      USHORT Type : 4;
        // } BASE_RELOCATION_ENTRY, *PBASE_RELOCATION_ENTRY;
        let relocation_block_entries_count = (relocation_block.SizeOfBlock as usize - core::mem::size_of::<IMAGE_BASE_RELOCATION>() as usize) / BaseRelocationEntry::c_size();
        // println!("relocation_block_entries_count {}", relocation_block_entries_count);
        // println!("relocation_block.VirtualAddress: {}", relocation_block.VirtualAddress);
        // println!("");
        // ---- Up to here things look right. ----

        let mut relocation_entry_ptr: *mut u16 = (relocation_block_ref as usize + core::mem::size_of::<IMAGE_BASE_RELOCATION>() as usize) as *mut u16;
        for _index in 1..relocation_block_entries_count {
            let relocation_entry: BaseRelocationEntry = BaseRelocationEntry::new(unsafe{*relocation_entry_ptr});
            if relocation_entry.reloc_type as u32 == IMAGE_REL_BASED_DIR64 || relocation_entry.reloc_type as u32 == IMAGE_REL_BASED_HIGHLOW {
                let addr_to_be_patched = (new_dll_base as usize + relocation_block.VirtualAddress as usize + relocation_entry.offset as usize) as *mut usize;
                let new_value_at_addr  = unsafe { *addr_to_be_patched } + image_base_delta as usize;
                unsafe { *addr_to_be_patched = new_value_at_addr };
            }
            // Unable to validate up to here but %40 confident this is working.
            // Big improvement over last iteration.
            relocation_entry_ptr = (relocation_entry_ptr as usize + BaseRelocationEntry::c_size()) as *mut u16;
        }
        relocation_block_ref = (relocation_block_ref as usize + relocation_block.SizeOfBlock as usize) as *mut IMAGE_BASE_RELOCATION;
    }
    // uiValueB = (ULONG_PTR)&((PIMAGE_NT_HEADERS)uiHeaderValue)->OptionalHeader.DataDirectory[ IMAGE_DIRECTORY_ENTRY_BASERELOC ];
    Ok(())
}

// AND the ILT entry (a 64 or 32 bit value) by the b10000000... to get the most signifacnt bit.
// Check if that most significant bit is 0 or 1. 
// If it's 1 then the function should be loaded by ordinal reference.   - return True
// If it's 0 then the function should be loaded by name.                - return False
fn image_snap_by_ordinal(ordinal: usize) -> bool{
    #[cfg(target_arch = "x86_64")]
    return (ordinal as u64 & IMAGE_ORDINAL_FLAG64) != 0;
    #[cfg(target_arch = "x86")]   
    return (ordinal as u32 & IMAGE_ORDINAL_FLAG32) != 0;
}

/// Extract the 0-15 bytes which represent the ordinal
/// reference to import the function with.
/// C variation: `def IMAGE_ORDINAL(Ordinal): return (Ordinal & 0xffff)`
fn image_ordinal(ordinal: usize) -> u16 {
    return (ordinal & 0xffff) as u16;
}


fn process_import_address_tables(new_dll_base: *mut c_void, pe_file_headers: &PeFileHeaders64) -> Result<()>{
    let import_directory: IMAGE_DATA_DIRECTORY = pe_file_headers.nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];
	
    if import_directory.Size == 0 {
        // No relocations to process
        return Ok(());
    }

    let mut base_image_import_table: *mut IMAGE_IMPORT_DESCRIPTOR = (new_dll_base as usize + import_directory.VirtualAddress as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    loop {
        let import_table_descriptor = unsafe{*base_image_import_table};
        if import_table_descriptor.Name == 0 {
            break;
        }
        // println!("NameRVA: {:#06x}", import_table_descriptor.Name);

        let slice = (new_dll_base as usize + import_table_descriptor.Name as usize) as *const i8;
        let library_name = unsafe { CStr::from_ptr(slice) };
        // println!("library_name: {}", library_name.to_str()?); // gotta cut the null terminated strings out.
        let library_handle = unsafe { LoadLibraryA( library_name.as_ptr() as *const u8) };
        // println!("library_handle: {:#08x}", library_handle);
        if library_handle != 0 {
            #[cfg(target_arch = "x86_64")]
            let mut library_thunk_ref = (new_dll_base as usize + import_table_descriptor.FirstThunk as usize) as *mut IMAGE_THUNK_DATA64;
            #[cfg(target_arch = "x86")]
            let mut library_first_thunk_ref = (new_dll_base as usize + import_table_descriptor.FirstThunk as usize) as *mut IMAGE_THUNK_DATA32;
            // println!("library_thunk_ref: {:#08x}", library_thunk_ref as usize);
            loop {
                // Simply dereferencing a pointer may result in the struct being copied instead of referenced.
                // let mut library_thunk: IMAGE_THUNK_DATA64 = unsafe { *library_thunk_ref };
                // Instead we need to dereference to a mutable reference.
                // We can't just set it equal since that will be a pointer to the object.
                // To use it each line would need to dereference the pointer then access the field.
                // let mut library_thunk: *mut IMAGE_THUNK_DATA64 = library_thunk_ref;
                let mut library_thunk = unsafe { &mut *library_thunk_ref };
                // println!("library_thunk_ref         {:p}", library_thunk_ref);
                // println!("library_thunk             {:p}", library_thunk);
                // println!("library_thunk.u1.Function {:p}", &unsafe{library_thunk.u1.Function});

                // Access of a union field is unsafe
                if unsafe{library_thunk.u1.AddressOfData} == 0 {
                    break;
                }
                // println!("library_thunk.u1.Function before: {:#08x}", unsafe{library_thunk.u1.Function});
                if image_snap_by_ordinal(unsafe{library_thunk.u1.Ordinal as usize}) {
                    // println!("Import by ordinal");
                    // Calculate the ordinal reference to the function from the library_thunk entry.
                    let function_ordinal = image_ordinal(unsafe{library_thunk.u1.Ordinal as usize}) as *const u8;
                    // println!("library_thunk.u1.Function: {:?}", unsafe{library_thunk.u1.Function});
                    // Get the address of the function using `GetProcAddress` and update the thunks reference.
                    library_thunk.u1.Function = unsafe { GetProcAddress(library_handle, function_ordinal).unwrap() as _};
                } else {
                    // println!("Import by name");
                    // Calculate a refernce to the function name by adding the dll_base and name's RVA.
                    let function_name_ref: *mut IMAGE_IMPORT_BY_NAME = (new_dll_base as usize + unsafe{library_thunk.u1.AddressOfData} as usize) as *mut IMAGE_IMPORT_BY_NAME;
                    // println!("(*function_name_ref).Name: {:?}", unsafe{CStr::from_ptr((*function_name_ref).Name.as_ptr() as *const i8)} );
                    // Get the address of the function using `GetProcAddress` and update the thunks reference.
                    // println!("library_thunk.u1.Function: {:#08x}", unsafe{library_thunk.u1.Function});
                    // debug_wait();
                    let tmp_new_func_addr = unsafe{ GetProcAddress(library_handle, (*function_name_ref).Name.as_ptr()).unwrap() as _};
                    // println!("tmp_new_func_addr: {:#08x}", tmp_new_func_addr); // This seems to point to the functions in the DLL.
                    library_thunk.u1.Function = tmp_new_func_addr;
                    // println!("library_thunk.u1.Function: {:#08x}", unsafe{library_thunk.u1.Function}); // This seems to get updated correctly.
                }
                // println!("library_thunk.u1.Function after:  {:#08x}", unsafe{library_thunk.u1.Function});
                library_thunk_ref = (library_thunk_ref as usize + core::mem::size_of::<usize>()) as *mut IMAGE_THUNK_DATA64;
            }
        }
        base_image_import_table = (base_image_import_table as usize + core::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>() as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    }

    Ok(())
}

#[no_mangle] // change to ptr to memory address.
pub fn reflective_loader(dll_bytes: *mut c_void) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!("This OS isn't supported by the dll_reflect function.\nOnly windows systems are supported"));

    #[cfg(target_arch = "x86_64")]
    let pe_header = PeFileHeaders64::new(dll_bytes)?;
    #[cfg(target_arch = "x86")]
    let pe_header = PeFileHeaders32::new(dll_bytes)?;

    if pe_header.dos_header.e_magic != 23117 {
        return Err(anyhow::anyhow!("DOS Header mismatch"));
    }
    // Allocate memory for our DLL to be loaded into
    let new_dll_base = unsafe { VirtualAlloc(ptr::null(), pe_header.nt_headers.OptionalHeader.SizeOfImage as usize, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
    
    // Write our DLL headers into the newly allocated memory.
    unsafe { core::ptr::copy(dll_bytes, new_dll_base, pe_header.nt_headers.OptionalHeader.SizeOfImage as usize) }

    // copy over DLL image sections to the newly allocated space for the DLL
    relocate_dll_image_sections(new_dll_base, dll_bytes as *const c_void, &pe_header)?;

    // Get distance between new dll memory and on disk image base.
    if pe_header.nt_headers.OptionalHeader.ImageBase as usize > new_dll_base as usize {
        return Err(anyhow::anyhow!("image_base ptr was greater than dll_mem ptr."));
    }
    let image_base = pe_header.nt_headers.OptionalHeader.ImageBase as usize;
    let image_base_delta = new_dll_base as usize - image_base;
    let entry_point = (new_dll_base as usize + pe_header.nt_headers.OptionalHeader.AddressOfEntryPoint as usize) as *const fnDllMain;
    let dll_main_func = unsafe { core::mem::transmute::<_, fnDllMain>(entry_point) };
    // println!("dll_main_func:   {:#08x}", dll_main_func as usize);
    // println!("entry_point:     {:#08x}", entry_point as usize);
    // println!("new_dll_base:    {:#08x}", new_dll_base as usize);
    // debug_wait(); // attach debugger seems to fail out when 

    // perform image base relocations
    process_dll_image_relocation(new_dll_base, &pe_header, image_base_delta)?;

	// resolve import address table
    process_import_address_tables(new_dll_base, &pe_header)?;

    // Execute DllMain
    unsafe{dll_main_func(new_dll_base as isize, DLL_PROCESS_ATTACH, 0 as *mut c_void);}

    Ok(())
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
    fn test_dll_reflect_new_base_relocation_entry() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let test_entry: u16 = 0xA148;
        let base_reloc_entry = BaseRelocationEntry::new(test_entry);
        assert_eq!(base_reloc_entry.offset, 0x148);
        assert_eq!(base_reloc_entry.reloc_type, 0xa);
        Ok(())
    }

    #[test]
    fn test_dll_reflect_parse_pe_headers() -> anyhow::Result<()>{
        
        // Get the path to our test dll file.
        let read_in_dll_bytes = include_bytes!("..\\..\\create_file_dll\\target\\debug\\create_file_dll.dll");
        let dll_bytes = read_in_dll_bytes.as_ptr() as *mut c_void;

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
        let expected_virtual_addr = vec![
            0x1000,
            0x1d000,
            0x26000,
            0x27000,
            0x29000,
        ];
        let expected_characteristics = vec![
            0x60000020,
            0x40000040,
            0xc0000040,
            0x40000040,
            0x42000040,
        ];
        for (section_index, section) in pe_file_headers.section_headers.iter().enumerate() {
            if section_index >= pe_file_headers.nt_headers.FileHeader.NumberOfSections as usize { break; }
            println!("{:?}", String::from_utf8(section.Name.to_vec())?);
            assert_eq!(expected_section_names[section_index], String::from_utf8(section.Name.to_vec())?);
            assert_eq!(expected_virtual_addr[section_index], section.VirtualAddress);
            assert_eq!(expected_characteristics[section_index], section.Characteristics);
        }
        Ok(())
    }

    #[test]
    fn test_dll_reflect_against_loadlibrarya() -> anyhow::Result<()>{
        let file_path = "..\\..\\create_file_dll\\target\\debug\\create_file_dll.dll";
        let module_base_addr = unsafe{ LoadLibraryA(file_path.as_ptr())};
        println!("module_base_addr: {:#08x}", module_base_addr);
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
        let read_in_dll_bytes = include_bytes!("..\\..\\create_file_dll\\target\\debug\\create_file_dll.dll");
        let dll_bytes = read_in_dll_bytes.as_ptr() as *mut c_void;

        // Set env var in our process cuz rn we only self inject.
        std::env::set_var("LIBTESTFILE", path.clone());
        // Run our code.
        let _res = reflective_loader(dll_bytes)?;

        let delay = time::Duration::from_secs(DLL_EXEC_WAIT_TIME);
        thread::sleep(delay);

        // Test that the test file was created
        let test_path = Path::new(path.as_str());
        assert!(test_path.is_file());

        // Delete test file
        let _ = fs::remove_file(test_path);
        Ok(())
    }
}

