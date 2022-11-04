use std::fs;

use anyhow::Result;
use tera::{Context, Tera};
use serde_json::Value;

fn build_context(json_data: String) -> Result<Context> {
    let serde_dict: Value = serde_json::from_str(json_data.as_str())?;
    let mut context = Context::new();
    for (key, value) in serde_dict.as_object().unwrap() {
        context.insert(key.as_str(), &value);
    }
    return Ok(context);
}

pub fn template(template_path: String, dst_path: String, args: String, autoescape: bool) -> Result<()> {
    let context = build_context(args)?;
    let template_content = fs::read_to_string(template_path)?;
    let res_content = Tera::one_off(template_content.as_str(), &context, autoescape)?;
    fs::write(dst_path, res_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_template_build_context() -> anyhow::Result<()>{
        let res = build_context(r#"
        {
            "name": "test123",
            "phone": "1237891",
            "folks": [
                { "name":"test" },
                { "name":"test2" }
            ]
        }"#.to_string());
        assert_eq!(format!("{:?}", res.unwrap()), r#"Context { data: {"folks": Array [Object {"name": String("test")}, Object {"name": String("test2")}], "name": String("test123"), "phone": String("1237891")} }"#);
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

        // Run our code
        template(path, dst_path.clone(), json_data, true)?;

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
        template(path, dst_path.clone(), json_data, true)?;
        
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
