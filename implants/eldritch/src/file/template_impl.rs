use std::fs;

use anyhow::Result;
use starlark::values::dict::FrozenDict;
use starlark::{collections::SmallMap};
use starlark::values::{Value, dict};
use tera::{Context, Tera};
// use serde_json::Value;

// fn build_context_json(json_data: String) -> Result<Context> {
//     let serde_dict: serde_json::Value = serde_json::from_str(json_data.as_str())?;
//     let mut context = Context::new();
//     for (key, value) in serde_dict.as_object().unwrap() {
//         context.insert(key.as_str(), &value);
//     }
//     return Ok(context);
// }

fn build_context(dict_data: SmallMap<String, Value>) -> Result<Context> {
    let mut context = Context::new();
    for (key, value) in dict_data {
        let tmp = value;
        context.insert(key.as_str(), &tmp);
    }
    return Ok(context);
}

pub fn template(template_path: String, dst_path: String, args: SmallMap<String, Value>, autoescape: bool) -> Result<()> {
    let tmp_dict = args.get("config").unwrap();
    println!("{:?}", tmp_dict);
    
    let context = build_context(args)?;
    let template_content = fs::read_to_string(template_path)?;
    let res_content = Tera::one_off(template_content.as_str(), &context, autoescape)?;
    fs::write(dst_path, res_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, hash::Hash, vec};
    use starlark::{collections::SmallMap, values::{ValueLike, Heap, dict::Dict, FrozenHeap, FrozenValue, FrozenStringValue, list::FrozenList}};
    use tempfile::NamedTempFile;
    use starlark::const_frozen_string;
    use starlark::collections::Hashed;

    use super::*;

    #[test]
    fn test_template_build_context() -> anyhow::Result<()>{
        let mut map: SmallMap<String, Value> = SmallMap::new();
        map.insert("name".to_string(), const_frozen_string!("greg").to_value());
        map.insert("age".to_string(), Value::new_int(29));
        map.insert("admin".to_string(), Value::new_bool(true));

        let res = build_context(map)?;
        assert_eq!(format!("{:?}", res), r#"Context { data: {"admin": Bool(true), "age": Number(29), "name": String("greg")} }"#);
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
        let mut dict_data: SmallMap<String, Value> = SmallMap::new();
        dict_data.insert("name".to_string(), const_frozen_string!("test123").to_value());
        dict_data.insert("age".to_string(), Value::new_int(22));

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
    fn test_template_loops_and_lists() -> anyhow::Result<()>{
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        let dst_tmp_file = NamedTempFile::new()?;
        let dst_path = String::from(tmp_file.path().to_str().unwrap());
        dst_tmp_file.close()?;

        // Write out template
        fs::write(path.clone(),
r#"All the animals on the farm:{% for animal in animals %}
-{{ animal }}{% endfor %}
"#.as_bytes())?;

        // Setup our args
        let mut dict_data: SmallMap<String, Value> = SmallMap::new();
        let animals_list = [const_frozen_string!("pig").to_frozen_value(), const_frozen_string!("cow").to_frozen_value()];
        let frozenHeap = FrozenHeap::new();
        let animal_heap = frozenHeap.alloc_list(&animals_list[0..2]);
        dict_data.insert("animals".to_string(), animal_heap.to_value());

        // Run our code
        template(path, dst_path.clone(), dict_data, true)?;

        // Verify output
        let res = fs::read_to_string(dst_path)?;
        assert_eq!(res,
r#"All the animals on the farm:
-pig
-cow
"#.to_string());
        Ok(())
    }

    
    #[test]
//     fn test_template_dicts() -> anyhow::Result<()>{
//         let tmp_file = NamedTempFile::new()?;
//         let path = String::from(tmp_file.path().to_str().unwrap());
//         let dst_tmp_file = NamedTempFile::new()?;
//         let dst_path = String::from(tmp_file.path().to_str().unwrap());
//         dst_tmp_file.close()?;

//         // Write out template
//         fs::write(path.clone(),
// r#"Config contents:
// {{ config['name'] }}
// {{ config['path'] }}
// "#.as_bytes())?;

//         // Setup our args
//         let mut dict_data: SmallMap<String, Value> = SmallMap::new();
//         //{"config": Value(DictGen(RefCell { value: Dict { content: {Value("name"): Value("nginx.config"), Value("path"): Value("/etc/ngnix.config")} } }))}
        

//         dict_data.insert("config".to_string(), config_heap.to_value());

//         // Run our code
//         template(path, dst_path.clone(), dict_data, true)?;

//         // Verify output
//         let res = fs::read_to_string(dst_path)?;
//         assert_eq!(res,
// r#"Config contents:
// nginx.config
// /etc/nginx/nginx.conf
// "#.to_string());
//         Ok(())
//     }

    #[test]
    fn test_template_nested_object() -> anyhow::Result<()>{
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

        // let json_data= r#"
        // {
        //     "jobs": [
        //         {
        //             "name":"test",
        //             "jobid":1
        //         },
        //         {
        //             "name":"test2",
        //             "jobid":2
        //         },
        //         {
        //             "name":"job3",
        //             "jobid":3
        //         }
        //     ]
        // }"#.to_string();

        let mut dict_data: SmallMap<String, Value> = SmallMap::new();

        // Try using records.
        // let mut job_num_one: SmallMap<String, Value> = SmallMap::new();
        // job_num_one.insert_hashed("name".to_string(), const_frozen_string!("test").to_value());
        // job_num_one.insert("jobid".to_string(), Value::new_int(1));
        // dict_data.insert("jobs".to_string(), job_num_one_heap.to_value());

        // Try using Dict object
        // let mut job_num_one: SmallMap<FrozenStringValue, Value> = SmallMap::new();
        // job_num_one.insert(const_frozen_string!("name"), const_frozen_string!("test").to_value());

        // let mut job_num_one_dict = Dict::new(job_num_one);
        // let frozenHeap = FrozenHeap::new();
        //  let job_num_one_heap = frozenHeap.alloc_simple(job_num_one);
        // dict_data.insert("jobs".to_string(), job_num_one_dict);



        // Try creating a frozen heap to hold the nested jobs.
        // let frozenHeap = FrozenHeap::new();
        // let job_num_one_heap = frozenHeap.alloc_simple(job_num_one);




        // let dict_num_one = Dict::new(SmallMap::from(job_num_one));
        // let dict_num_two = Dict::new(SmallMap::new());
        // dict_num_one.insert_hashed( Hashed::new(Hash::new()), const_frozen_string!("test").to_value());


        // let mut job_num_two: SmallMap<String, Value> = SmallMap::new();
        // job_num_two.insert("name".to_string(), const_frozen_string!("test2").to_value());
        // job_num_two.insert("jobid".to_string(), Value::new_int(2));
        // let mut job_num_three: SmallMap<String, Value> = SmallMap::new();
        // job_num_three.insert("name".to_string(), const_frozen_string!("job3").to_value());
        // job_num_three.insert("jobid".to_string(), Value::new_int(3));


//         let heap = Heap::new();
//         // let vect = vec![job_num_one, job_num_two, job_num_three];
//         let vect = vec![dict_num_one, dict_num_two];
        // let dict_val = heap.alloc((job_num_one, true));

//         dict_data.insert("jobs".to_string(), dict_val );

//         println!("{:?}", dict_data);

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
