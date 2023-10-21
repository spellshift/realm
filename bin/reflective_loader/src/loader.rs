use ntapi::{ntpsapi::PEB_LDR_DATA, ntldr::LDR_DATA_TABLE_ENTRY, ntpebteb::PEB};
use windows_sys::{Win32::{System::{Memory::{VIRTUAL_ALLOCATION_TYPE, PAGE_PROTECTION_FLAGS}, Diagnostics::Debug::{IMAGE_DIRECTORY_ENTRY_BASERELOC, IMAGE_DATA_DIRECTORY, IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_SECTION_HEADER_0, IMAGE_DIRECTORY_ENTRY_EXPORT, IMAGE_OPTIONAL_HEADER64}, SystemServices::{IMAGE_BASE_RELOCATION, IMAGE_IMPORT_DESCRIPTOR, IMAGE_ORDINAL_FLAG64, IMAGE_IMPORT_BY_NAME, IMAGE_REL_BASED_DIR64, IMAGE_REL_BASED_HIGHLOW, DLL_PROCESS_ATTACH, IMAGE_EXPORT_DIRECTORY}, WindowsProgramming::IMAGE_THUNK_DATA64}, Foundation::{HINSTANCE, BOOL, FARPROC}}, core::PCSTR};
use windows_sys::Win32::{System::{Diagnostics::Debug::{IMAGE_NT_HEADERS64,IMAGE_SECTION_HEADER},SystemServices::{IMAGE_DOS_HEADER},Memory::{MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE}}};
use core::{ffi::CStr, arch::asm, slice::from_raw_parts, mem::transmute, ptr::null_mut};
use core::ptr;
use core::ffi::c_void;

 // This is an assumption that may be wrong. Due to no_std it's ideal to keep things
 // on the stack which has a default size of 1024KB. If more sections are needed
 // we can rework this hopefully by then a no_std zero-copy pe parser will exist
 // and our home grown PE parser can be removed.
const MAX_PE_SECTIONS: usize = 32;
const PE_MAGIC: u16 = 0x5A4D; // MZ
const NT_SIGNATURE: u32 = 0x4550; // PE


const KERNEL32_HASH: u32 = 0x6ddb9555;
const NTDLL_HASH: u32 = 0x1edab0ed;
const LOAD_LIBRARY_A_HASH: u32 = 0xb7072fdb;
const GET_PROC_ADDRESS_HASH: u32 = 0xdecfc1bf;
const VIRTUAL_ALLOC_HASH: u32 = 0x97bc257;
const GET_LAST_ERROR_HASH: u32  = 0x8160BDC3;

#[derive(Copy,Clone)]
struct UserData {
    function_offset: u64,
}

type FnDllMain = unsafe extern "system" fn(module: HINSTANCE, call_reason: u32, reserved: *mut c_void) -> BOOL;

#[allow(non_camel_case_types)]
type generic_fn = unsafe extern "system" fn() -> ();

type FnLoadLibraryA = unsafe extern "system" fn(lplibfilename: PCSTR) -> HINSTANCE;

type FnGetProcAddress = unsafe extern "system" fn(hmodule: HINSTANCE, lpprocname: PCSTR) -> FARPROC;

type FnVirtualAlloc = unsafe extern "system" fn(lpaddress: *const c_void, dwsize: usize, flallocationtype: VIRTUAL_ALLOCATION_TYPE, flprotect: PAGE_PROTECTION_FLAGS) -> *mut c_void;

type FnGetLastError = unsafe extern "system" fn() -> u32;

// pub unsafe fn VirtualAlloc(hprocess: super::super::Foundation::HANDLE, lpaddress: *const ::core::ffi::c_void, dwsize: usize, flallocationtype: VIRTUAL_ALLOCATION_TYPE, flprotect: PAGE_PROTECTION_FLAGS) -> *mut ::core::ffi::c_void
fn virtual_alloc(fn_ptr: FnVirtualAlloc, err_ptr: FnGetLastError, lp_address: *const c_void, dw_size: usize, fl_allocation_type: u32, fl_protect: u32) -> *mut c_void {
    let buffer_handle: *mut c_void = unsafe{ fn_ptr(lp_address, dw_size, fl_allocation_type, fl_protect) };
    if buffer_handle == null_mut() {
        let error_code = unsafe { err_ptr() };
        if error_code != 0 {
            panic!("Failed to allocate memory. Last error returned: {}", error_code);
        }
    }
    buffer_handle
}

// pub unsafe fn GetProcAddress(hmodule: P0, lpprocname: P1) -> FARPROC
fn get_proc_address(fn_ptr: FnGetProcAddress, err_ptr: FnGetLastError, h_module: isize, lp_proc_name: *const u8) -> isize {
    let proc_handle = match unsafe { fn_ptr(h_module, lp_proc_name) } {
        Some(local_proc_handle) => local_proc_handle,
        None => {
            let error_code = unsafe { err_ptr() };
            panic!("Failed to find function {:?} in module. Last error returned: {}", lp_proc_name, error_code);
        }
    };
    proc_handle as isize
}


// pub unsafe fn LoadLibraryA(lplibfilename: P0) ->  Result<HMODULE>
fn load_library_a(fn_ptr: FnLoadLibraryA, err_ptr: FnGetLastError, lplibfilename: PCSTR) -> HINSTANCE {
    let library_handle = unsafe { fn_ptr(lplibfilename) };
    if library_handle == 0 {
        let error_code = unsafe { err_ptr() };
        if error_code != 0 {
            panic!("Failed to load library. Last error returned: {}", error_code);
        }
    }
    library_handle
}

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
            offset,
            reloc_type,
        }
    }

    fn c_size() -> usize {
        core::mem::size_of::<u16>()
    }
}

#[cfg(target_arch = "x86_64")]
struct PeFileHeaders64 {
    dos_headers: IMAGE_DOS_HEADER,
    nt_headers: IMAGE_NT_HEADERS64,
    section_headers: [IMAGE_SECTION_HEADER; MAX_PE_SECTIONS],
}

// Pares the PE file from a series of bytes
#[cfg(target_arch = "x86_64")]
impl PeFileHeaders64 {
    fn new(dll_bytes_ptr: *mut c_void) -> Self{
        // DOS Headers
        let dos_headers_base_ptr = dll_bytes_ptr as usize;
        let dos_headers = unsafe { *((dos_headers_base_ptr) as *mut IMAGE_DOS_HEADER) };
        if dos_headers.e_magic != PE_MAGIC {
            panic!("PE Magic header mismatch. Expected 0x5A4D == MZ == 21117. File does not appear to be a PE executable.");
        }

        // NT Headers
        let nt_headers_base_ptr = dos_headers_base_ptr + dos_headers.e_lfanew as usize;
        let nt_headers = unsafe { *((nt_headers_base_ptr) as *mut IMAGE_NT_HEADERS64) };
        if nt_headers.Signature != NT_SIGNATURE {
            panic!("NT Signature mismatch. Expected 0x4550 == PE == 17744. File does not appear to be a PE executable.");
        }

        // Section Headers - hopefully there isn't more than MAX_PE_SECTIONS sections.
        let null_section = IMAGE_SECTION_HEADER {
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
        let mut section_headers: [IMAGE_SECTION_HEADER; MAX_PE_SECTIONS] = [null_section; MAX_PE_SECTIONS];
        // let mut section_headers: [IMAGE_SECTION_HEADER; MAX_PE_SECTIONS] = unsafe{ core::mem::zeroed() };
        let optional_headers_start_ptr = unsafe{&(*(nt_headers_base_ptr as *mut IMAGE_NT_HEADERS64)).OptionalHeader as *const _ as usize};
        let section_headers_start_ptr = optional_headers_start_ptr + nt_headers.FileHeader.SizeOfOptionalHeader as usize;
        let mut cur_section_ptr = section_headers_start_ptr as *mut IMAGE_SECTION_HEADER;
        for section_index in 0..nt_headers.FileHeader.NumberOfSections {
            let cur_section = unsafe { *cur_section_ptr.clone() };

            section_headers[section_index as usize] = cur_section;

            cur_section_ptr =
                    (cur_section_ptr as usize + core::mem::size_of::<IMAGE_SECTION_HEADER>() as usize) as *mut IMAGE_SECTION_HEADER 
        }    

        Self {
            dos_headers,
            nt_headers,
            section_headers,
        }
    }
}

/// In order to keep the compiler from being upset about
/// not being able to find memmove or memcpy we need to 
/// implement our own copy function that doesn't call etiher.
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        let local_src = unsafe { src.add(i) };
        let local_dest = unsafe { dest.add(i) };
        unsafe{*local_dest = *local_src};
    }
    dest
}

// #[no_mangle]
// pub unsafe extern "C" fn memset(dest: *mut u8, val: u8, n: usize) -> () {
//     core::intrinsics::volatile_set_memory(dest, val, n)
// }


/// Copy each DLL section into the newly allocated memory.
/// Each section is copied according to it's VirtualAddress.
fn relocate_dll_image_sections(new_dll_base: *mut c_void, old_dll_bytes: *const c_void, pe_file_headers: &PeFileHeaders64) -> () {
    for (section_index, section) in pe_file_headers.section_headers.iter().enumerate() {
        if section_index >= pe_file_headers.nt_headers.FileHeader.NumberOfSections as usize { return; } 
        let section_destination = new_dll_base as usize + section.VirtualAddress as usize;
        let section_bytes = old_dll_bytes as usize + section.PointerToRawData as usize;
        
        unsafe { memcpy(section_destination as *mut u8, section_bytes as *const u8, section.SizeOfRawData as usize) };
    }
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
fn process_dll_image_relocation(new_dll_base: *mut c_void, pe_file_headers: &PeFileHeaders64, image_base_delta: isize) -> () {
    let relocation_directory: IMAGE_DATA_DIRECTORY = pe_file_headers.nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC as usize];	
    if relocation_directory.Size == 0 {
        // No relocations to process
        return;
    }
    let mut relocation_block_ref: *mut IMAGE_BASE_RELOCATION = 
        (new_dll_base as usize + relocation_directory.VirtualAddress as usize) as *mut IMAGE_BASE_RELOCATION;
    loop {
        let relocation_block = unsafe{*relocation_block_ref as IMAGE_BASE_RELOCATION};
        if relocation_block.SizeOfBlock == 0 ||
            relocation_block.VirtualAddress == 0 {
            break;
        }

        // This needs to be calculated since the relocation_block doesn't track it.
        // Luckily the relocation_entry is a static size: u16.
        // Unfortunately the struct uses offset bits which is annoying in Rust.
        // c++ struct:
        // typedef struct BASE_RELOCATION_ENTRY {
        //      USHORT Offset : 12;
        //      USHORT Type : 4;
        // } BASE_RELOCATION_ENTRY, *PBASE_RELOCATION_ENTRY;
        let relocation_block_entries_count = (relocation_block.SizeOfBlock as usize - core::mem::size_of::<IMAGE_BASE_RELOCATION>() as usize) / BaseRelocationEntry::c_size();
        let mut relocation_entry_ptr: *mut u16 = (relocation_block_ref as usize + core::mem::size_of::<IMAGE_BASE_RELOCATION>() as usize) as *mut u16;
        for _index in 0..relocation_block_entries_count {
            let relocation_entry: BaseRelocationEntry = BaseRelocationEntry::new(unsafe{*relocation_entry_ptr});
            if relocation_entry.reloc_type as u32 == IMAGE_REL_BASED_DIR64 || relocation_entry.reloc_type as u32 == IMAGE_REL_BASED_HIGHLOW {
                let addr_to_be_patched = (new_dll_base as usize + relocation_block.VirtualAddress as usize + relocation_entry.offset as usize) as *mut usize;
                let new_value_at_addr  = unsafe { *addr_to_be_patched } + image_base_delta as usize;
                unsafe { *addr_to_be_patched = new_value_at_addr };
            }
            relocation_entry_ptr = (relocation_entry_ptr as usize + BaseRelocationEntry::c_size()) as *mut u16;
        }
        relocation_block_ref = (relocation_block_ref as usize + relocation_block.SizeOfBlock as usize) as *mut IMAGE_BASE_RELOCATION;
    }
}

/// AND the ILT entry (a 64 or 32 bit value) by the b10000000... to get the most signifacnt bit.
/// Check if that most significant bit is 0 or 1. 
/// If it's 1 then the function should be loaded by ordinal reference.   - return True
/// If it's 0 then the function should be loaded by name.                - return False
fn image_snap_by_ordinal(ordinal: usize) -> bool{
    #[cfg(target_arch = "x86_64")]
    return (ordinal as u64 & IMAGE_ORDINAL_FLAG64) != 0;
}

/// Extract the 0-15 bytes which represent the ordinal
/// reference to import the function with.
/// C variation: `def IMAGE_ORDINAL(Ordinal): return (Ordinal & 0xffff)`
fn image_ordinal(ordinal: usize) -> u16 {
    return (ordinal & 0xffff) as u16;
}

fn process_import_address_tables(new_dll_base: *mut c_void, pe_file_headers: &PeFileHeaders64, load_library_a_fn: FnLoadLibraryA, get_proc_address_fn: FnGetProcAddress, get_last_error_fn: FnGetLastError) -> () {
    let import_directory: IMAGE_DATA_DIRECTORY = pe_file_headers.nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];
	
    if import_directory.Size == 0 {
        // No relocations to process
        return;
    }

    let mut base_image_import_table: *mut IMAGE_IMPORT_DESCRIPTOR = (new_dll_base as usize + import_directory.VirtualAddress as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    loop {
        let import_table_descriptor = unsafe{*base_image_import_table};
        if import_table_descriptor.Name == 0 {
            break;
        }

        let library_name_ptr = (new_dll_base as usize + import_table_descriptor.Name as usize) as *const i8;
        let library_name = unsafe { CStr::from_ptr(library_name_ptr) };
        let library_handle = load_library_a(load_library_a_fn, get_last_error_fn, library_name.as_ptr() as *const u8);
        if library_handle != 0 {
            #[cfg(target_arch = "x86_64")]
            let mut library_thunk_ptr = (new_dll_base as usize + import_table_descriptor.FirstThunk as usize) as *mut IMAGE_THUNK_DATA64;
            loop {
                // Simply dereferencing a pointer may result in the struct being copied instead of referenced.
                // let mut library_thunk: IMAGE_THUNK_DATA64 = unsafe { *library_thunk_ref };
                // Instead we need to dereference to a mutable reference.
                // We can't just set it equal since that will be a pointer to the object.
                // To use it each line would need to dereference the pointer then access the field.
                // let mut library_thunk: *mut IMAGE_THUNK_DATA64 = library_thunk_ref;
                let mut library_thunk = unsafe { &mut *library_thunk_ptr };

                // Access of a union field is unsafe
                if unsafe{library_thunk.u1.AddressOfData} == 0 {
                    break;
                }
                if image_snap_by_ordinal(unsafe{library_thunk.u1.Ordinal as usize}) {
                    // Calculate the ordinal reference to the function from the library_thunk entry.
                    let function_ordinal_ptr = image_ordinal(unsafe{library_thunk.u1.Ordinal as usize}) as *const u8;
                    // Get the address of the function using `GetProcAddress` and update the thunks reference.
                    library_thunk.u1.Function = get_proc_address(get_proc_address_fn, get_last_error_fn ,library_handle, function_ordinal_ptr) as _;
                } else {
                    // Calculate a refernce to the function name by adding the dll_base and name's RVA.
                    let image_import_ptr: *mut IMAGE_IMPORT_BY_NAME = (new_dll_base as usize + unsafe{library_thunk.u1.AddressOfData} as usize) as *mut IMAGE_IMPORT_BY_NAME;
                    // Get the address of the function using `GetProcAddress` and update the thunks reference.
                    let function_name_ptr = unsafe {(*image_import_ptr ).Name.as_ptr()};
                    // let tmp_new_func_addr = unsafe{ get_proc_address_fn(library_handle, function_name).unwrap() as _};
                    let tmp_new_func_addr = get_proc_address(get_proc_address_fn, get_last_error_fn, library_handle, function_name_ptr) as _;
                    library_thunk.u1.Function = tmp_new_func_addr;
                }
                library_thunk_ptr = (library_thunk_ptr as usize + core::mem::size_of::<usize>()) as *mut IMAGE_THUNK_DATA64;
            }
        }
        base_image_import_table = (base_image_import_table as usize + core::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>() as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    }

}


/// Get a pointer to the Process Environment Block (PEB)
pub unsafe fn get_peb() -> *mut PEB 
{
    let peb: *mut PEB;
    asm!("mov {peb}, gs:[0x60]", peb = out(reg) peb);
    peb
}

/// Generate a unique hash
pub fn dbj2_hash(buffer: &[u8]) -> u32 
{
    let mut hsh: u32 = 5381;
    let mut iter: usize = 0;
    let mut cur: u8;

    while iter < buffer.len() 
    {
        cur = buffer[iter];

        if cur == 0 
        {
            iter += 1;
            continue;
        }

        if cur >= ('a' as u8) 
        {
            cur -= 0x20;
        }

        hsh = ((hsh << 5).wrapping_add(hsh)) + cur as u32;
        iter += 1;
    }

    return hsh;
}

/// Get loaded module by hash
pub unsafe fn get_loaded_module_by_hash(module_hash: u32) -> Option<*mut u8> 
{
    let peb = get_peb();
    let peb_ldr_data_ptr = (*peb).Ldr as *mut PEB_LDR_DATA;
    let mut module_list = (*peb_ldr_data_ptr).InLoadOrderModuleList.Flink as *mut LDR_DATA_TABLE_ENTRY;

    while !(*module_list).DllBase.is_null() 
    {
        let dll_buffer_ptr = (*module_list).BaseDllName.Buffer;
        let dll_length = (*module_list).BaseDllName.Length as usize;
        let dll_name_slice = from_raw_parts(dll_buffer_ptr as *const u8, dll_length);

        if module_hash == dbj2_hash(dll_name_slice) 
        {
            return Some((*module_list).DllBase as _);
        }

        module_list = (*module_list).InLoadOrderLinks.Flink as *mut LDR_DATA_TABLE_ENTRY;
    }

    return None;
}

/// Get the address of an export by hash
unsafe fn get_export_by_hash(module_base: *mut u8, export_name_hash: u32) -> Option<usize>
{
    let pe_file = PeFileHeaders64::new(module_base as *mut c_void);
    let nt_headers = pe_file.nt_headers;
    let export_directory = (module_base as usize + nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT as usize].VirtualAddress as usize) as *mut IMAGE_EXPORT_DIRECTORY;
    let names = from_raw_parts((module_base as usize + (*export_directory).AddressOfNames as usize) as *const u32, (*export_directory).NumberOfNames as _);
    let functions = from_raw_parts((module_base as usize + (*export_directory).AddressOfFunctions as usize) as *const u32, (*export_directory).NumberOfFunctions as _,);
    let ordinals = from_raw_parts((module_base as usize + (*export_directory).AddressOfNameOrdinals as usize) as *const u16, (*export_directory).NumberOfNames as _);

    for i in 0..(*export_directory).NumberOfNames 
    {
        let name_addr = (module_base as usize + names[i as usize] as usize) as *const i8;
        let name_len = get_cstr_len(name_addr as _);
        let name_slice: &[u8] = from_raw_parts(name_addr as _, name_len);

        if export_name_hash == dbj2_hash(name_slice) 
        {
            let ordinal = ordinals[i as usize] as usize;
            return Some(module_base as usize + functions[ordinal] as usize);
        }
    }

    return None;
}

/// Get the length of a C String
pub unsafe fn get_cstr_len(pointer: *const char) -> usize 
{
    let mut tmp: u64 = pointer as u64;

    while *(tmp as *const u8) != 0 
    {
        tmp += 1;
    }

    (tmp - pointer as u64) as _
}

#[no_mangle]
pub fn reflective_loader(user_data_ptr_and_dll_bytes: *mut c_void) -> usize {
    #[cfg(not(target_os = "windows"))]
    panic!("This OS isn't supported by the dll_reflect function.\nOnly windows systems are supported");

    // Dereference user data object
    let user_data_ptr_ptr = user_data_ptr_and_dll_bytes as *const *const UserData;
    let user_data_ptr = unsafe{ *user_data_ptr_ptr };
    let user_data = unsafe{*user_data_ptr}.clone();
    if user_data.function_offset == 0 { panic!("Could not parse the user_data segment") }
    
    // Increment dll_bytes ptr offset to after the user_data ptr.
    let dll_bytes = (user_data_ptr_and_dll_bytes as usize + core::mem::size_of::<usize>()) as *mut c_void;

    #[cfg(target_arch = "x86_64")]
    let pe_header = PeFileHeaders64::new(dll_bytes);
    if pe_header.dos_headers.e_magic != PE_MAGIC { panic!("Target DLL does not appear to be a DLL.") }

    let kernel32_base = unsafe { get_loaded_module_by_hash(KERNEL32_HASH).unwrap() };
    let ntdll_base = unsafe { get_loaded_module_by_hash(NTDLL_HASH).unwrap() };
    if kernel32_base.is_null() || ntdll_base.is_null() 
    {
        panic!("Could not find kernel32 and ntdll");
    }

    // Create function pointers
    // Get exports
    let loadlib_addy = unsafe { get_export_by_hash(kernel32_base, LOAD_LIBRARY_A_HASH).expect("Couldn't lookup LoadLibraryA export by hash") };
    let load_library_a_fn = unsafe { transmute::<_, FnLoadLibraryA>(loadlib_addy) };

    let getproc_addy = unsafe { get_export_by_hash(kernel32_base, GET_PROC_ADDRESS_HASH).expect("Couldn't lookup GetProcAddress export by hash") };
    let get_proc_address_fn = unsafe { transmute::<_, FnGetProcAddress>(getproc_addy) };

    let virtualalloc_addy = unsafe { get_export_by_hash(kernel32_base, VIRTUAL_ALLOC_HASH).expect("Couldn't lookup VirtualAlloc export by hash") };
    let virtual_alloc_fn = unsafe { transmute::<_, FnVirtualAlloc>(virtualalloc_addy) };

    let getlasterror_addy = unsafe { get_export_by_hash(kernel32_base, GET_LAST_ERROR_HASH).expect("Couldn't lookup GetLastError export by hash") };
    let get_last_error_fn = unsafe{ transmute::<_, FnGetLastError>(getlasterror_addy)};

    // Allocate memory for our DLL to be loaded into
    let new_dll_base: *mut c_void = virtual_alloc(virtual_alloc_fn, get_last_error_fn, ptr::null(), pe_header.nt_headers.OptionalHeader.SizeOfImage as usize, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE);

    // // copy over DLL image sections to the newly allocated space for the DLL
    relocate_dll_image_sections(new_dll_base, dll_bytes as *const c_void, &pe_header); // This uses memcpy which is unresolved

    // Get distance between new dll memory and on disk image base.
    let image_base_delta = new_dll_base as isize - pe_header.nt_headers.OptionalHeader.ImageBase as isize;

    // perform image base relocations
    process_dll_image_relocation(new_dll_base, &pe_header, image_base_delta);
    // resolve import address table
    process_import_address_tables(new_dll_base, &pe_header, load_library_a_fn, get_proc_address_fn, get_last_error_fn);

    // Execute DllMain
    let entry_point = (new_dll_base as usize + pe_header.nt_headers.OptionalHeader.AddressOfEntryPoint as usize) as *const FnDllMain;
    let dll_main_func = unsafe { core::mem::transmute::<_, FnDllMain>(entry_point) };
    unsafe{dll_main_func(new_dll_base as isize, DLL_PROCESS_ATTACH, 0 as *mut c_void);}

    // Call the function the user specified.
    let user_entry_point = (new_dll_base as usize + user_data.function_offset as usize ) as *const generic_fn;
    let user_entry_point_fn = unsafe{ core::mem::transmute::<_, generic_fn>(user_entry_point)};
    unsafe{user_entry_point_fn()};

    new_dll_base as usize
}


#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use super::*;
    use core::{time};
    use std::{thread, path::{Path, PathBuf}, fs};
    use object::{LittleEndian, read::pe::{ImageThunkData}, pe::ImageNtHeaders64, Object, ObjectSection};
    use tempfile::NamedTempFile;
    use windows_sys::Win32::{System::{Memory::VirtualAlloc, LibraryLoader::LoadLibraryA}, Foundation::GetLastError};

    const TEST_PAYLOAD: &[u8] = include_bytes!("..\\..\\create_file_dll\\target\\debug\\create_file_dll.dll");
    const TEST_PAYLOAD_RELATIVE_PATH: &str = "..\\create_file_dll\\target\\debug\\create_file_dll.dll";

    fn get_export_address_by_name(pe_bytes: &[u8], export_name: &str, in_memory: bool) -> anyhow::Result<usize> {
        let pe_file = object::read::pe::PeFile64::parse(pe_bytes)?;
    
        let section = match pe_file.section_by_name(".text") {
            Some(local_section) => local_section,
            None => return Err(anyhow::anyhow!(".text section not found")),
        };
    
        let mut section_raw_data_ptr = 0x0;
        for section in pe_file.section_table().iter() {
            let section_name = String::from_utf8(section.name.to_vec())?;
            if section_name.contains(".text") {
                section_raw_data_ptr = section.pointer_to_raw_data.get(LittleEndian);
                break;
            }
        }
        if section_raw_data_ptr == 0x0 {
            return Err(anyhow::anyhow!("Failed to find pointer to text section."))
        }
    
        // Section offset for .text.
        let rva_offset = section.address() as usize - section_raw_data_ptr as usize - pe_file.relative_address_base() as usize;
    
        let exported_functions = pe_file.exports()?;
        for export in exported_functions {
            if export_name == String::from_utf8(export.name().to_vec())?.as_str() {
                if in_memory {
                    return Ok(export.address() as usize - pe_file.relative_address_base() as usize);
                } else {
                    return Ok(export.address() as usize - rva_offset - pe_file.relative_address_base() as usize);
                }
            }
        }
    
        Err(anyhow::anyhow!("Function {} not found", export_name))
    }    

    #[test]
    fn test_reflective_loader_memcpy_simple() -> () {
        let source_buffer = [0,1,2,3,4];
        let mut dest_buffer = [0,0,0,0,0];
        unsafe { memcpy(dest_buffer.as_mut_ptr(), source_buffer.as_ptr(), source_buffer.len()) };
        for (index, byte) in source_buffer.iter().enumerate() {
            assert_eq!(*byte, dest_buffer[index]);
        }
    }

    #[test]
    fn test_reflective_loader_memcpy_overlapping_fwd() -> () {
        let dest_offset = 3;
        let source_offset = 0;
        let common_buffer = [0,1,2,3,4,5,6,7,8,9,10];
        let expected_output = common_buffer.clone();

        let source_buffer = (common_buffer.as_ptr() as usize + source_offset as usize) as *mut u8;
        let dest_buffer = (common_buffer.as_ptr() as usize + dest_offset as usize) as *mut u8;

        unsafe { memcpy(dest_buffer, source_buffer, common_buffer.len() - (dest_offset + source_offset)) };
        for index in 0..common_buffer.len() - (dest_offset + source_offset) { 
            assert_eq!(unsafe{*dest_buffer.add(index)}, expected_output[index+source_offset])
        }
    }

    #[test]
    fn test_reflective_loader_memcpy_overlapping_rev() -> () {
        let dest_offset = 0;
        let source_offset = 3;
        let common_buffer = [0,1,2,3,4,5,6,7,8,9,10];
        let expected_output = common_buffer.clone();

        let source_buffer = (common_buffer.as_ptr() as usize + source_offset as usize) as *mut u8;
        let dest_buffer = (common_buffer.as_ptr() as usize + dest_offset as usize) as *mut u8;

        unsafe { memcpy(dest_buffer, source_buffer, common_buffer.len() - (dest_offset + source_offset)) };
        for index in 0..common_buffer.len() - (dest_offset + source_offset) {
            assert_eq!(unsafe{*dest_buffer.add(index)}, expected_output[index+source_offset])
        }
    }

    #[test]
    fn test_reflective_loader_get_export_by_hash() -> () {
        // Try getting the function pointer
        let kernel32_hash = KERNEL32_HASH;
        let virtual_alloc_hash = VIRTUAL_ALLOC_HASH;
        let kernel32_base = unsafe { get_loaded_module_by_hash(kernel32_hash).unwrap() };
        let virtualalloc_addy = unsafe { get_export_by_hash(kernel32_base, virtual_alloc_hash).unwrap() };
        assert!(virtualalloc_addy > 0);
        // Try calling the function
        #[allow(non_camel_case_types)]
        type fnVirtualAlloc = unsafe extern "system" fn(lpaddress: *const c_void, dwsize: usize, flallocationtype: VIRTUAL_ALLOCATION_TYPE, flprotect: PAGE_PROTECTION_FLAGS) -> *mut c_void;    
        let virtual_alloc = unsafe { transmute::<_, fnVirtualAlloc>(virtualalloc_addy) };
        let res = unsafe{virtual_alloc(core::ptr::null(), 1024, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)};
        assert!(res as usize > 0 );
    }

    #[test]
    fn test_reflective_loader_get_module_by_hash() -> () {
        let kernel32_hash = KERNEL32_HASH;
        let kernel32_base = unsafe { get_loaded_module_by_hash(kernel32_hash).unwrap() };
        assert!(kernel32_base as usize > 0);
    }
    
    #[test]
    fn test_reflective_loader_dbj2_hash() -> () {
        let test_names = [
            "kernel32.dll".as_bytes(),
            "ntdll.dll".as_bytes(),
            "LoadLibraryA".as_bytes(),
            "GetProcAddress".as_bytes(),
            "VirtualAlloc".as_bytes(),
            "GetLastError".as_bytes(),
        ];
        let test_hashes = [
            KERNEL32_HASH,
            NTDLL_HASH,
            LOAD_LIBRARY_A_HASH,
            GET_PROC_ADDRESS_HASH,
            VIRTUAL_ALLOC_HASH,
            GET_LAST_ERROR_HASH,
        ];
        for (index, name) in test_names.iter().enumerate() {
            let expected = test_hashes[index];
            let res = dbj2_hash(&name);
            assert_eq!(res, expected);
        }
    }

    #[test]
    fn test_reflective_loader_new_base_relocation_entry_low() -> () {
        // Get the path to our test dll file.
        let test_entry: u16 = 0xA148;
        let base_reloc_entry = BaseRelocationEntry::new(test_entry);
        assert_eq!(base_reloc_entry.offset, 0x148);
        assert_eq!(base_reloc_entry.reloc_type, 0xa);
    }

    #[test]
    fn test_reflective_loader_new_base_relocation_entry_medium() -> () {
        // Get the path to our test dll file.
        let test_entry: u16 = 0xA928;
        let base_reloc_entry = BaseRelocationEntry::new(test_entry);
        assert_eq!(base_reloc_entry.offset, 0x928);
        assert_eq!(base_reloc_entry.reloc_type, 0xa);
    }

    #[test]
    fn test_reflective_loader_new_base_relocation_entry_high() -> () {
        // Get the path to our test dll file.
        let test_entry: u16 = 0xAFA8;
        let base_reloc_entry = BaseRelocationEntry::new(test_entry);
        assert_eq!(base_reloc_entry.offset, 0xFA8);
        assert_eq!(base_reloc_entry.reloc_type, 0xa);
    }

    // PE Headers change everytime create file dll is built
    // #[test]
    // fn test_reflective_loader_parse_pe_headers() -> () {
        
    //     // Get the path to our test dll file.
    //     let read_in_dll_bytes = TEST_PAYLOAD;
    //     let dll_bytes = read_in_dll_bytes.as_ptr() as *mut c_void;

    //     let pe_file_headers = PeFileHeaders64::new(dll_bytes);
    //     //get_dos_headers(dll_bytes.as_ptr() as usize)?;
    //     // 0x5A4D == a"ZM" == d23117 --- PE Magic number is static.
    //     assert_eq!(PE_MAGIC, pe_file_headers.dos_headers.e_magic);
    //     // 0x020B == d523
    //     assert_eq!(NT_SIGNATURE, pe_file_headers.nt_headers.Signature);

    //     let expected_section_names = vec![
    //         ".text\0\0\0",
    //         ".rdata\0\0",
    //         ".data\0\0\0",
    //         ".pdata\0\0",
    //         ".reloc\0\0",
    //     ];
    //     let expected_virtual_addr = vec![
    //         0x1000,
    //         0x1d000,
    //         0x26000,
    //         0x27000,
    //         0x29000,
    //     ];
    //     let expected_characteristics = vec![
    //         0x60000020,
    //         0x40000040,
    //         0xc0000040,
    //         0x40000040,
    //         0x42000040,
    //     ];
    //     for (section_index, section) in pe_file_headers.section_headers.iter().enumerate() {
    //         if section_index >= pe_file_headers.nt_headers.FileHeader.NumberOfSections as usize { break; }
    //         assert_eq!(expected_section_names[section_index], String::from_utf8(section.Name.to_vec()).unwrap());
    //         assert_eq!(expected_virtual_addr[section_index], section.VirtualAddress);
    //         assert_eq!(expected_characteristics[section_index], section.Characteristics);
    //     }
    // }

    #[test]
    fn test_reflective_loader_simple() -> () {
        const DLL_EXEC_WAIT_TIME: u64 = 3;
        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new().unwrap();
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close().unwrap();

        // Get the path to our test dll file.
        let read_in_dll_bytes = TEST_PAYLOAD;

        // Create user_data struct and ptr bytes
        let user_data = UserData{
            function_offset: get_export_address_by_name(read_in_dll_bytes, "demo_init", true).unwrap() as u64,
        };
        let user_data_ptr = (&user_data as *const _) as usize;
        let user_data_ptr_bytes = user_data_ptr.to_le_bytes();
        let user_data_ptr_as_slice = user_data_ptr_bytes.as_slice();
        

        // let dll_bytes_and_user_data = [user_data_ptr_slice, read_in_dll_bytes.try_into().unwrap()].concat().as_slice();
        let user_data_ptr_and_dll_bytes_vec = [user_data_ptr_as_slice,read_in_dll_bytes].concat();
        let user_data_ptr_and_dll_bytes = user_data_ptr_and_dll_bytes_vec.as_slice().as_ptr() as *mut c_void;

        // Set env var in our process.
        std::env::set_var("LIBTESTFILE", path.clone());
        // Run our code.
        let _res = reflective_loader(user_data_ptr_and_dll_bytes);

        let delay = time::Duration::from_secs(DLL_EXEC_WAIT_TIME);
        thread::sleep(delay);

        // Test that the test file was created
        let test_path = Path::new(path.as_str());
        assert!(test_path.is_file());

        // Delete test file
        let _ = fs::remove_file(test_path);
    }

    // Compare the relocated bytes from our reflective_loader and
    // LoadLibraryA function. Using object library to parse the PE
    // to remove our parsing as a potential error.
    #[test]
    fn test_reflective_loader_process_dll_image_relocation() -> anyhow::Result<()> {
        let mut test_payload_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_payload_path.push(TEST_PAYLOAD_RELATIVE_PATH);

        // Get the path to our test dll file.
        let read_in_dll_bytes = TEST_PAYLOAD;
        let dll_bytes = read_in_dll_bytes.as_ptr() as *mut c_void;

        let pe_header = PeFileHeaders64::new(dll_bytes);
        // Allocate memory for our DLL to be loaded into
        let test_dll_base: *mut c_void = unsafe { VirtualAlloc(ptr::null(), pe_header.nt_headers.OptionalHeader.SizeOfImage as usize, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
        // copy over DLL image sections to the newly allocated space for the DLL
        relocate_dll_image_sections(test_dll_base, dll_bytes as *const c_void, &pe_header); // This uses memcpy which is unresolved
        let image_base_delta = test_dll_base as isize - pe_header.nt_headers.OptionalHeader.ImageBase as isize;
        process_dll_image_relocation(test_dll_base, &pe_header, image_base_delta);

        let good_dll_base = unsafe{ LoadLibraryA(format!("{}\0", test_payload_path.as_path().to_str().unwrap()).as_ptr()) };
        if good_dll_base == 0 {
            let last_err = unsafe{GetLastError()};
            return Err(anyhow::anyhow!("Failed to load test DLL with `LoadLibraryA` check that the file exists. Last error: {}", last_err));
        }
        // Parse bytes from disk.
        let pe_file = object::read::pe::PeFile64::parse(read_in_dll_bytes)?;
        let section_table = pe_file.section_table();
        let good_image_base_delta = good_dll_base - pe_file.nt_headers().optional_header.image_base.get(LittleEndian) as isize;

        // Loop over the relocations and check against the updated dll bytes.
        let mut blocks = pe_file.data_directories().relocation_blocks(read_in_dll_bytes, &section_table)?.unwrap();
        while let Some(block) = blocks.next()? {
            for reloc in block {
                let test_addr = (test_dll_base as usize + reloc.virtual_address as usize) as *mut usize;
                if test_addr as usize > test_dll_base as usize + pe_header.nt_headers.OptionalHeader.SizeOfImage as usize { panic!("About to read out of bounds in test") }

                let known_good_addr = (good_dll_base as usize + reloc.virtual_address as usize) as *mut usize;
                if known_good_addr as usize > good_dll_base as usize + pe_header.nt_headers.OptionalHeader.SizeOfImage as usize { panic!("About to read out of bounds in known good") }

                assert_eq!((unsafe{*test_addr} as usize - image_base_delta as usize), (unsafe{*known_good_addr} as usize - good_image_base_delta as usize));
            }
        }
        Ok(())
    }

    // Compare the import bytes from our reflective_loader and
    // LoadLibraryA function. Using object library to parse the PE
    // to remove our parsing as a potential error. Checks that the
    // imports reference points to the same function that LoadLibrary
    // would set it to.
    #[test]
    fn test_reflective_loader_process_import_address_tables() -> anyhow::Result<()> {
        let mut test_payload_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_payload_path.push(TEST_PAYLOAD_RELATIVE_PATH);

        // Get the path to our test dll file.
        let read_in_dll_bytes = TEST_PAYLOAD;
        let dll_bytes = read_in_dll_bytes.as_ptr() as *mut c_void;

        let pe_header = PeFileHeaders64::new(dll_bytes);
        // Allocate memory for our DLL to be loaded into
        let test_dll_base: *mut c_void = unsafe { VirtualAlloc(ptr::null(), pe_header.nt_headers.OptionalHeader.SizeOfImage as usize, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
        // copy over DLL image sections to the newly allocated space for the DLL
        relocate_dll_image_sections(test_dll_base, dll_bytes as *const c_void, &pe_header); // This uses memcpy which is unresolved
        let image_base_delta = test_dll_base as isize - pe_header.nt_headers.OptionalHeader.ImageBase as isize;
        process_dll_image_relocation(test_dll_base, &pe_header, image_base_delta);
        let good_dll_base = unsafe{ LoadLibraryA(format!("{}\0", test_payload_path.as_path().to_str().unwrap()).as_ptr()) };
        if good_dll_base == 0 {
            let last_err = unsafe{GetLastError()};
            return Err(anyhow::anyhow!("Failed to load test DLL with `LoadLibraryA` check that the file exists. Last error: {}", last_err));
        }
        // Parse bytes from disk.
        let pe_file = object::read::pe::PeFile64::parse(read_in_dll_bytes)?;
        let section_table = pe_file.section_table();

        if let Some(import_table) = pe_file.data_directories().import_table(read_in_dll_bytes, &section_table)? {
            let mut import_descs = import_table.descriptors()?;
            while let Some(import_desc) = import_descs.next()? {    
                let lookup_thunks = import_table.thunks(import_desc.original_first_thunk.get(LittleEndian))?;

                let mut thunks = lookup_thunks.clone();
                while let Some(thunk) = thunks.next::<ImageNtHeaders64>()? {
                    let good_first_few_fn_bytes = unsafe{*((thunk.address() as usize + good_dll_base as usize) as *const usize)};
                    let test_first_few_fn_bytes = unsafe{*((thunk.address() as usize + test_dll_base as usize) as *const usize)};
                    assert_eq!(test_first_few_fn_bytes, good_first_few_fn_bytes);
                }
            }
        }

        Ok(())
    }

}

