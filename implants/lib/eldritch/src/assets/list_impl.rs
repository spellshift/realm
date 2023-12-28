use anyhow::Result;

pub fn list() -> Result<Vec<String>> {
    let mut res: Vec<String> = Vec::new();
    for file_path in super::Asset::iter() {
        res.push(file_path.to_string());
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets_list() -> anyhow::Result<()> {
        let res_all_embedded_files = list()?;

        assert_eq!(
            res_all_embedded_files,
            [
                "exec_script/hello_world.bat",
                "exec_script/hello_world.sh",
                "exec_script/main.eldritch",
                "print/main.eldritch"
            ]
        );

        Ok(())
    }
}
