use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;


pub fn append(_path: String, _content: String) -> Result<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)    //Do we want to create the file if it doesn't exist?
        .append(true)
        .open(_path)
        .unwrap();

    if let Err(e) = writeln!(file, "{}", _content) {
        eprintln!("Couldn't write to file: {}", e);
        return Err(e)?
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::fs::File;
    use std::fs::remove_file;

    #[test]
    fn test_append_existing() {
        //Remove the test file
        let _rmfileres = remove_file(String::from("/tmp/win"));
        // Run  our code
        let _res = append(String::from("/tmp/win"), String::from("Hi2!"));
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
        let _rmfileres = remove_file(String::from("/tmp/win"));

    
    }
}
