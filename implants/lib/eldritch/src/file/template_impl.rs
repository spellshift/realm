use std::fs;

use anyhow::Result;
use starlark::collections::SmallMap;
use starlark::values::Value;
use tera::{Context, Tera};

fn build_context(dict_data: SmallMap<String, Value>) -> Result<Context> {
    let mut context = Context::new();
    for (key, value) in dict_data {
        let tmp = value;
        context.insert(key.as_str(), &tmp);
    }
    Ok(context)
}

pub fn template(
    template_path: String,
    dst_path: String,
    args: SmallMap<String, Value>,
    autoescape: bool,
) -> Result<()> {
    let context = build_context(args)?;
    let template_content = fs::read_to_string(template_path)?;
    let res_content = Tera::one_off(template_content.as_str(), &context, autoescape)?;
    fs::write(dst_path, res_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use starlark::const_frozen_string;
    use starlark::{
        collections::SmallMap,
        values::{FrozenHeap, Heap},
    };
    use std::fs;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_template_build_context() -> anyhow::Result<()> {
        let heap = Heap::new();
        let mut map: SmallMap<String, Value> = SmallMap::new();
        map.insert("name".to_string(), const_frozen_string!("greg").to_value());
        map.insert("age".to_string(), heap.alloc(29));
        map.insert("admin".to_string(), Value::new_bool(true));

        let res = build_context(map)?;
        assert_eq!(
            format!("{:?}", res),
            r#"Context { data: {"admin": Bool(true), "age": Number(29), "name": String("greg")} }"#
        );
        Ok(())
    }

    #[test]
    fn test_template_basic() -> anyhow::Result<()> {
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        let dst_tmp_file = NamedTempFile::new()?;
        let dst_path = String::from(tmp_file.path().to_str().unwrap());
        dst_tmp_file.close()?;

        let heap = Heap::new();

        // Write out template
        fs::write(
            path.clone(),
            r#"Hello {{ name }},
welcome to the eldritch realm test.
I see you're {{ age }} years old.
Congratulations on making it that far.
"#
            .as_bytes(),
        )?;
        // Setup our args
        let mut dict_data: SmallMap<String, Value> = SmallMap::new();
        dict_data.insert(
            "name".to_string(),
            const_frozen_string!("test123").to_value(),
        );
        dict_data.insert("age".to_string(), heap.alloc(22));

        // Run our code
        template(path, dst_path.clone(), dict_data, true)?;

        // Verify output
        let res = fs::read_to_string(dst_path)?;
        assert_eq!(
            res,
            r#"Hello test123,
welcome to the eldritch realm test.
I see you're 22 years old.
Congratulations on making it that far.
"#
            .to_string()
        );
        Ok(())
    }

    #[test]
    fn test_template_loops_and_lists() -> anyhow::Result<()> {
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        let dst_tmp_file = NamedTempFile::new()?;
        let dst_path = String::from(tmp_file.path().to_str().unwrap());
        dst_tmp_file.close()?;

        // Write out template
        fs::write(
            path.clone(),
            r#"All the animals on the farm:{% for animal in animals %}
-{{ animal }}{% endfor %}
"#
            .as_bytes(),
        )?;

        // Setup our args
        let mut dict_data: SmallMap<String, Value> = SmallMap::new();
        let animals_list = vec!["pig".to_string(), "cow".to_string()];
        let frozen_heap = FrozenHeap::new();
        let animal_heap = frozen_heap.alloc(animals_list);

        dict_data.insert("animals".to_string(), animal_heap.to_value());

        // Run our code
        template(path, dst_path.clone(), dict_data, true)?;

        // Verify output
        let res = fs::read_to_string(dst_path)?;
        assert_eq!(
            res,
            r#"All the animals on the farm:
-pig
-cow
"#
            .to_string()
        );
        Ok(())
    }
}
