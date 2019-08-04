use std::fs;
use std::io::{Error,ErrorKind};
use std::io::BufReader;
use std::io::prelude::*;
use std::result::Result;

use serde_yaml;

use crate::{mode,util};


pub fn import_all_modes(filename: Option<&str>, verbose: bool) -> Result<Vec<mode::CvtMode>, Error> {
    let f = util::filename_or_default(filename,verbose)?;
    let file = fs::OpenOptions::new().write(true).read(true).create(true).open(f)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let modes = match serde_yaml::from_str(&contents) {
        Ok(m) => m,
        Err(_) => Vec::<mode::CvtMode>::new()

    };
    if verbose {
        for mode in &modes {
            println!("Found mode {:?}", mode);
        }
    }
    Ok(modes)
}

pub fn save_mode(mode: &mode::CvtMode, filename: Option<&str>, verbose: bool) -> Result<(), Error> {
    let f = util::filename_or_default(filename,verbose)?;
    let mut v = import_all_modes(filename,verbose)?;
    let n = mode.get_name();
    for i in 0 .. v.len() {
        if v[i].get_name() == n {
            if verbose {
                println!("Mode {} already exists in file {}. Overwriting.",n,f.to_str().unwrap());
            }
            v.remove(i);
        }
    }
    v.push(mode.clone());
    let s = serde_yaml::to_string(&v).unwrap();
    if verbose {
        println!("Writing to {}",f.to_str().unwrap());
    }
    // truncating here deletes the existing content of the file and replaces it with the mode saved/
    let mut file = fs::OpenOptions::new().write(true).truncate(true).open(f)?;
    file.write_all(s.as_bytes())?;
    Ok(())
}

pub fn get_mode(name: &str, filename: Option<&str>, verbose: bool) -> Result<mode::CvtMode, Error> {
    let modes = import_all_modes(filename, verbose)?;
    for m in modes {
        if m.get_name() == name {
            return Ok(m)
        }
    }
    Err(Error::new(ErrorKind::NotFound, "Mode not found."))
}

pub fn save_mode_persistent(_mode: &mode::CvtMode, _verbose: bool) -> Result<(), Error> {
    if cfg!(target_os = "linux") {
        let _file = fs::OpenOptions::new().append(true).open("$HOME/.xprofile")?;
    }
    Ok(())
}
