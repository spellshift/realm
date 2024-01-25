use anyhow::Result;

use super::AssetsLibrary;

pub fn list(this: AssetsLibrary) -> Result<Vec<String>> {
    // println!("this:{:?}", this.0);
    let mut res: Vec<String> = Vec::new();
    res.push(format!("{:?}", this.0));
    for file_path in super::Asset::iter() {
        res.push(file_path.to_string());
    }

    Ok(res)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_assets_list() -> anyhow::Result<()> {
//         let res_all_embedded_files = list(AssetsLibrary("test123".to_string()))?;

//         assert_eq!(
//             res_all_embedded_files,
//             [
//                 "exec_script/hello_world.bat",
//                 "exec_script/hello_world.sh",
//                 "exec_script/main.eldritch",
//                 "print/main.eldritch"
//             ]
//         );

//         Ok(())
//     }
// }
