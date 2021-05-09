use std::path::PathBuf;
use std::fs;

pub fn list_files(path: &str) -> Vec<PathBuf> {
    let dir = fs::read_dir(path).expect("can not read folder");
    let files = dir.map(|res| res.map(|e| e.path()));
    let mut paths = vec!();
    for file_buff in files {
        match file_buff {
            Ok(b) => paths.push(b),
            Err(e) => println!("failed to read a path due to {:?}", e)
        }
    }

    paths
}