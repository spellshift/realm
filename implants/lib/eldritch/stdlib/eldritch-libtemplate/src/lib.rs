extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;
#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("template")]
/// The `template` library renders Jinja2 templates from strings and files.
pub trait TemplateLibrary {
    #[eldritch_method]
    /// Renders a template string using the supplied variables.
    ///
    /// `autoescape` enables HTML escaping of substituted values.
    fn render(
        &self,
        template: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<String, String>;

    #[eldritch_method]
    /// Reads a template file, renders it, and writes the result to `dst`.
    ///
    /// Returns an error if the template cannot be read, parsed, or written.
    fn render_file(
        &self,
        template_path: String,
        dst: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<(), String>;
}
