use copy_dir::*;

use crate::error::*;

use std::ffi::OsStr;
use std::fs::{create_dir_all, read_dir};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub fn slurp<T: AsRef<Path>>(path: T) -> Result<String> {
    use std::fs::File;
    use std::io::prelude::*;
    let mut buf = String::new();
    let mut file = File::open(&path).ctx(&path.as_ref())?;
    file.read_to_string(&mut buf).ctx(&path.as_ref())?;
    Ok(buf)
}

/// Write `contents` into `path` tries to create the directories if they
/// don't exist yet
pub fn spit<A: AsRef<Path>>(path: A, contents: &str) -> Result<()> {
    use std::fs::OpenOptions;
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        create_dir_all(&parent).ctx(parent)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .ctx(&path)?;
    Ok(file.write_all(contents.as_bytes()).ctx(path)?)
}

pub fn contents_of_directory<A: AsRef<Path>>(
    directory: A,
    file_type: &str,
) -> Result<Vec<PathBuf>> {
    let valid_type = OsStr::new(file_type);
    let mut matches: Vec<PathBuf> = Vec::new();
    let directory_path = directory.as_ref();
    for entry in read_dir(directory_path).ctx(directory_path)? {
        let entry = entry.ctx(directory_path)?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        match path.extension() {
            Some(x) if x == valid_type => matches.push(path),
            _ => {
                println!("Invalid file: {:?}", &path);
                continue;
            }
        };
    }
    Ok(matches)
}

pub fn clear_directory<A: AsRef<Path>>(directory: A) -> Result<()> {
    let directory_path = directory.as_ref();
    for entry in read_dir(directory_path).ctx(directory_path)? {
        let entry = entry.ctx(directory_path)?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path).ctx(&path)?;
        } else {
            std::fs::remove_file(&path).ctx(&path)?;
        }
    }
    Ok(())
}

pub fn copy_items_to_directory<A: AsRef<Path>>(
    items: &Vec<String>,
    from_dir: A,
    to_dir: A,
) -> Result<()> {
    for entry in items {
        let source = from_dir.as_ref().join(entry);
        if !source.exists() {
            println!("Could not find path {:?}", &entry);
            continue;
        };
        let target = to_dir.as_ref().join(entry);
        println!("copy '{:?}' to '{:?}'", &source, &target);
        // We copy each item seperately, so we can see when it fails
        match copy_dir(&source, &target) {
            Ok(ref e) if e.len() > 0 => e.iter().for_each(|_| println!("Could not copy {:?}", &e)),
            Err(e) => println!("Copy Error: {:?}", &e),
            _ => (),
        };
    }
    Ok(())
}
