use super::TemplateLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

pub mod render_file_impl;
pub mod render_impl;

#[derive(Debug, Default)]
#[eldritch_library_impl(TemplateLibrary)]
pub struct StdTemplateLibrary;

impl TemplateLibrary for StdTemplateLibrary {
    fn render(
        &self,
        template: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<String, String> {
        render_impl::render(template, args, autoescape).map_err(|e| e.to_string())
    }

    fn render_file(
        &self,
        template_path: String,
        dst: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<(), String> {
        render_file_impl::render_file(template_path, dst, args, autoescape)
    }
}
