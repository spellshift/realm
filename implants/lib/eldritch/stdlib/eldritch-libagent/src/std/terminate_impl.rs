use alloc::string::String;

pub fn terminate() -> Result<(), String> {
    ::std::process::exit(0);
}
