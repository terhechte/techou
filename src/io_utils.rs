use copy_dir::*;

use std::path::{Path, PathBuf};
use std::fs::{read_dir, create_dir_all};
use std::ffi::OsStr;
use std::io::prelude::*;
use std::fs::File;

use crate::error::*;

pub fn slurp<T: AsRef<Path>>(path: T) -> Result<String> {
    use std::fs::File;
    use std::io::prelude::*;
    let mut buf = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

/// Write `contents` into `path` tries to create the directories if they
/// don't exist yet
pub fn spit<A: AsRef<Path>>(path: A, contents: &str) -> Result<()> {
    use std::fs::OpenOptions;
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        create_dir_all(&parent)?;
    }
    let mut file = OpenOptions::new().write(true)
        .create_new(true)
        .open(path)?;
    Ok(file.write_all(contents.as_bytes())?)
}

pub fn contents_of_directory<A: AsRef<Path>>(directory: A, file_type: &str) -> Result<Vec<PathBuf>> {
    let valid_type = OsStr::new(file_type);
    let mut matches: Vec<PathBuf> = Vec::new();
    for entry in read_dir(directory.as_ref())? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() { continue }
        match path.extension() {
            Some(x) if x == valid_type => matches.push(path),
            _ => {
                println!("Invalid file: {:?}", &path);
                continue
            }
        };
    }
    Ok(matches)
}

pub fn clear_directory<A: AsRef<Path>>(directory: A) -> Result<()> {
    for entry in read_dir(directory.as_ref())? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn copy_items_to_directory<A: AsRef<Path>>(items: &Vec<String>, from_dir: A, to_dir: A) -> Result<()> {
    for entry in items {
        let path = from_dir.as_ref().join(entry);
        if !path.exists() {
            println!("Could not find path {:?}", &entry);
            continue
        };
        println!("copy '{:?}' to '{:?}'", &path, &to_dir.as_ref());
        // We copy each item seperately, so we can see when it fails
        match copy_dir(&path, &to_dir.as_ref()) {
            Ok(ref e) if e.len() > 0 => e.iter().for_each(|m| println!("Could not copy {:?}", &e)),
            Err(e) => println!("Copy Error: {:?}", &e),
            _ => ()
        };
    }
    Ok(())
}