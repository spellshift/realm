#![allow(clippy::mutable_key_type)]
#![allow(unexpected_cfgs)]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("sys")]
pub trait SysLibrary {
    #[eldritch_method]
    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    fn dll_reflect(
        &self,
        dll_bytes: Vec<u8>,
        pid: i64,
        function_name: String,
    ) -> Result<(), String>;

    #[eldritch_method]
    fn exec(
        &self,
        path: String,
        args: Vec<String>,
        disown: Option<bool>,
        env_vars: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn get_env(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String>;

    #[eldritch_method]
    fn get_os(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    fn get_pid(&self) -> Result<i64, String>;

    #[eldritch_method]
    fn get_reg(&self, reghive: String, regpath: String)
        -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    fn get_user(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn hostname(&self) -> Result<String, String>;

    #[eldritch_method]
    fn is_bsd(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn is_linux(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn is_macos(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn is_windows(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn write_reg_hex(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: String,
    ) -> Result<bool, String>;

    #[eldritch_method]
    fn write_reg_int(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: i64,
    ) -> Result<bool, String>;

    #[eldritch_method]
    fn write_reg_str(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: String,
    ) -> Result<bool, String>;
}
