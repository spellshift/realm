use super::PortalLibrary;
use alloc::string::String;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(PortalLibrary)]
pub struct PortalLibraryFake;

impl PortalLibrary for PortalLibraryFake {
    fn create(&self) -> Result<(), String> {
        Ok(())
    }
}
