use std::fs;

use anyhow::Result;
use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap};
use starlark::{collections::SmallMap};
use starlark::values::{Value, FrozenValue};
use tera::{Context, Tera};
// use serde_json::Value;



fn build_context_json(json_data: String) -> Result<Context> {
    let serde_dict: serde_json::Value = serde_json::from_str(json_data.as_str())?;
    let mut context = Context::new();
    for (key, value) in serde_dict.as_object().unwrap() {
        context.insert(key.as_str(), &value);
    }
    return Ok(context);
}

fn build_context(dict_data: SmallMap<String, Value>) -> Result<Context> {
    let mut context = Context::new();
    // for (key, value) in dict_data {
    //     println!("{}", value.get_type() );
    //     if value.get_type() == "string" {
    //         let tmp = String::unpack_value(value);
    //         context.insert(key.as_str(), &tmp);
    //     } else if value.get_type() == "int" {
    //         let tmp = i32::unpack_value(value);
    //         context.insert(key.as_str(), &tmp);
    //     } else if value.get_type() == "bool" {
    //         let tmp = bool::unpack_value(value);
    //         context.insert(key.as_str(), &tmp);
    //     } else {
    //         let tmp = json!(value);
    //         context.insert(key.as_str(), &tmp);
    //     }
    // }
    for (key, value) in dict_data {
        let tmp = value.serialize();
        context.insert(key.as_str(), &tmp);
    }
    return Ok(context);
}

pub fn template(template_path: String, dst_path: String, args: SmallMap<String, Value>, autoescape: bool) -> Result<()> {
    let context = build_context(args)?;
    let template_content = fs::read_to_string(template_path)?;
    let res_content = Tera::one_off(template_content.as_str(), &context, autoescape)?;
    fs::write(dst_path, res_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use starlark::smallmap;
    use starlark::values::list::ListOf;
    use tempfile::NamedTempFile;
    use starlark::const_frozen_string;
    use starlark::values::{FrozenStringValue, FrozenValue};
    
    use super::*;

    #[test]
    fn test_template_build_context() -> anyhow::Result<()>{
        let map: SmallMap<String, Value> = smallmap! {
            "name".to_string() => Value::new_frozen(const_frozen_string!("greg").unpack()),
            "age".to_string() => Value::new_int(29),
            "admin".to_string() => Value::new_bool(true),
        };
        let res = build_context(map)?;
        println!("{:?}", res);
        Ok(())
    }

    #[test]
    fn test_template_basic() -> anyhow::Result<()>{
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        let dst_tmp_file = NamedTempFile::new()?;
        let dst_path = String::from(tmp_file.path().to_str().unwrap());
        dst_tmp_file.close()?;

        // Write out template
        fs::write(path.clone(),
r#"Hello {{ name }},
welcome to the eldritch realm test.
I see you're {{ age }} years old.
Congratulations on making it that far.
"#.as_bytes())?;
        // Setup our args
        let json_data= r#"
        {
            "name": "test123",
            "age": 22
        }"#.to_string();

        let dict_data: SmallMap<String, Value> = smallmap! {};

        // Run our code
        template(path, dst_path.clone(), dict_data, true)?;

        // Verify output
        let res = fs::read_to_string(dst_path)?;
        assert_eq!(res, 
r#"Hello test123,
welcome to the eldritch realm test.
I see you're 22 years old.
Congratulations on making it that far.
"#.to_string());
        Ok(())
    }


    #[test]
    fn test_template_loops() -> anyhow::Result<()>{
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        let dst_tmp_file = NamedTempFile::new()?;
        let dst_path = String::from(tmp_file.path().to_str().unwrap());
        dst_tmp_file.close()?;

        // Write to file
        fs::write(path.clone(),
r#"name,jobid{% for job in jobs %}
{{ job['name'] }},{{ job['jobid'] }}{% endfor %}
"#.as_bytes())?;

        let json_data= r#"
        {
            "jobs": [
                { 
                    "name":"test",
                    "jobid":1
                },
                { 
                    "name":"test2",
                    "jobid":2
                },
                { 
                    "name":"job3",
                    "jobid":3
                }
            ]
        }"#.to_string();

        let dict_data = smallmap! {};

        template(path, dst_path.clone(), dict_data, true)?;
        
        let res = fs::read_to_string(dst_path)?;
        assert_eq!(res, 
r#"name,jobid
test,1
test2,2
job3,3
"#.to_string());
        Ok(())
    }
}
