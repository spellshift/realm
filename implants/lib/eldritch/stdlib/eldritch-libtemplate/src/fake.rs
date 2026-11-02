use super::TemplateLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Debug, Default)]
#[eldritch_library_impl(TemplateLibrary)]
pub struct TemplateLibraryFake;

impl TemplateLibrary for TemplateLibraryFake {
    fn render(
        &self,
        _template: String,
        _args: BTreeMap<String, Value>,
        _autoescape: bool,
    ) -> Result<String, String> {
        Ok(String::new())
    }

    fn render_file(
        &self,
        _template_path: String,
        _dst: String,
        _args: BTreeMap<String, Value>,
        _autoescape: bool,
    ) -> Result<(), String> {
        Ok(())
    }
}
