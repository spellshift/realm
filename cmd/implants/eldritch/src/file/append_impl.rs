use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;


pub fn append(path: String, content: String) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true) //Do we want to create the file if it doesn't exist? - Yes!
        .write(true)    
        .append(true)
        .open(path)?;

    writeln!(file, "{}", content)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::fs::File;
    use std::fs::remove_file;

    #[test]
    fn test_append_nonexisting() -> anyhow::Result<()> {
        //Remove the test file
        let _ = remove_file(String::from("/tmp/win"));
        // Run  our code
        append(String::from("/tmp/win"), String::from("Hi2!"))?;
        // Read the file
        let file = BufReader::new(File::open("/tmp/win").unwrap());
        // Reverse the file lines
        let mut lines: Vec<_> = file.lines().map(|line| { line.unwrap() }).collect();
        lines.reverse();

        //Make sure not empty
        assert_eq!((lines.len() > 0), true);
        
        // Make sure the last line equals == Hi2!
        for line in lines.iter() {
            println!("{}", line);
            assert_eq!(line, "Hi2!");
            // Last line so just break.
            break;
        }
        remove_file(String::from("/tmp/win"))?;
        Ok(())
    }    
    #[test]
    fn test_append_existing() -> anyhow::Result<()> {
        //Remove the test file
        let _remove_res = remove_file(String::from("/tmp/win"));
        let mut file = File::create("/tmp/win").unwrap();
        file.write_all(b"Hello, world!\n")?;

        // Run  our code
        append(String::from("/tmp/win"), String::from("Hi2!"))?;
        // Read the file
        let file = BufReader::new(File::open("/tmp/win").unwrap());
        // Reverse the file lines
        let mut lines: Vec<_> = file.lines().map(|line| { line.unwrap() }).collect();
        lines.reverse();

        //Make sure not empty
        assert_eq!((lines.len() > 0), true);
        
        // Make sure the last line equals == Hi2!
        for line in lines.iter() {
            println!("{}", line);
            assert_eq!(line, "Hi2!");
            // Last line so just break.
            break;
        }
        remove_file(String::from("/tmp/win"))?;
        Ok(())
    }
}
