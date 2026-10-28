#[cfg(feature = "stdlib")]
use alloc::collections::BTreeMap;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use eldritch_core::Value;

#[cfg(feature = "stdlib")]
pub fn render_file(
    template_path: String,
    dst: String,
    args: BTreeMap<String, Value>,
    autoescape: bool,
) -> Result<(), String> {
    render_file_impl(template_path, dst, args, autoescape).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn render_file(
    _template_path: alloc::string::String,
    _dst: alloc::string::String,
    _args: alloc::collections::BTreeMap<alloc::string::String, eldritch_core::Value>,
    _autoescape: bool,
) -> Result<(), alloc::string::String> {
    Err("render_file requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn render_file_impl(
    template_path: String,
    dst: String,
    args: BTreeMap<String, Value>,
    autoescape: bool,
) -> AnyhowResult<()> {
    use std::fs;

    let data = fs::read(&template_path)?;
    let template_content = String::from_utf8_lossy(&data);

    let res_content = super::render_impl::render(template_content.into_owned(), args, autoescape)?;
    fs::write(&dst, res_content)?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_template() {
        let tmp_tmpl = NamedTempFile::new().unwrap();
        let tmpl_path = tmp_tmpl.path().to_string_lossy().to_string();

        fs::write(&tmpl_path, "Hello {{ name }}").unwrap();

        let tmp_out = NamedTempFile::new().unwrap();
        let out_path = tmp_out.path().to_string_lossy().to_string();

        let mut args = BTreeMap::new();
        args.insert("name".to_string(), Value::String("World".to_string()));

        render_file(tmpl_path, out_path.clone(), args, true).unwrap();

        assert_eq!(fs::read_to_string(&out_path).unwrap(), "Hello World");
    }
}
