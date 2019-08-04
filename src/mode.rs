use std::{io,process,str,thread,time};
use std::io::Error;
use std::result::Result;
use regex::Regex;
use serde::{Serialize,Deserialize};
use crate::{fileio,util};

#[derive(Debug)]
pub struct InputMode {
    width:String,
    height:String,
    rate:String,
    name:String,
    display:String,
}


impl InputMode {
    pub fn new(width:&str,height:&str,rate:&str,display:&str,name:&str) -> InputMode {
        InputMode {
            width:width.to_string(),
            height:height.to_string(),
            rate:rate.to_string(),
            display:display.to_string(),
            name:name.to_string()
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct CvtMode {
    name: String,
    clock: String,
    h_disp: String,
    h_sync_start: String,
    h_sync_end: String,
    h_total: String,
    v_disp: String,
    v_sync_start: String,
    v_sync_end: String,
    v_total: String,
    flags: String,
}

impl CvtMode {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /*
    pub fn new_empty() -> CvtMode {
        CvtMode {
            name: String::new(),
            clock: String::new(),
            h_disp: String::new(),
            h_sync_start: String::new(),
            h_sync_end: String::new(),
            h_total: String::new(),
            v_disp: String::new(),
    v_sync_start: String::new(),
            v_sync_end: String::new(),
            v_total: String::new(),
            flags: String::new(),
        }
    }
    */
}



// Some(d) would be a vec of the displays for which to delete the mode; if d is None, the mode will be removed from all connected displays
// xrandr doesn't seem to think the program has access to user-created modes for deletion;
// could run as root but would rather not.
// TODO: address deletion permission issue
/*
fn delete_mode_xrandr(n: &str, d: Option<Vec<String>>, verbose: bool) -> Result<(),Error> {

    for display in d.unwrap() {
        delete_mode(&n,&display);
    }
    let currents_handle = thread::spawn(move || get_current_modes(verbose));
    let defaults_handle = thread::spawn(move || get_default_modes(verbose));
    let currents = currents_handle.join().unwrap()?;
    let defaults = defaults_handle.join().unwrap()?;
    let displays = match d {
        Some(disps) => disps,
        None => {
            let mut tmp: Vec<String> = Vec::with_capacity(currents.len());
            for mode in &currents {
                tmp.push(mode.display.clone());
            }
            tmp
        }
    };
    println!("{:?}",&currents);
    // these loops are because xrandr doesn't let you update modes or delete them while in use
    for disp in displays {
        for default in &defaults {
            if default.display == disp {
                if verbose {
                    println!("Switching to default mode to allow updating of the current mode");
                }
                switch_mode(&default.name, &disp, verbose)?; // switch the display to its default mode to enable deletion of in-use mode
            }
        }
        if verbose {
            println!("Removing mode {} from display {}",&n,&disp);
        }
        let mut cmd = process::Command::new("xrandr");
        cmd.arg("--delmode").arg(disp.clone()).arg(n.clone());
        println!("{:?}",cmd.output().unwrap());
    }
    Ok(())
}
*/

pub fn add_mode(w: Option<&str>, h: Option<&str>, r: Option<&str>, d: Option<&str>, n: Option<&str>, t: Option<&str>, f: Option<&str>, test: bool, save: bool, verbose: bool) -> Result<(),Error> {
    let current_modes = get_current_modes(verbose)?;
    // Use first current display mode for parameters not supplied
    // and as the fallback if test option is used
    let width = w.unwrap_or(&current_modes[0].width).to_string();
    let height = h.unwrap_or(&current_modes[0].height).to_string();
    let rate = r.unwrap_or(&current_modes[0].rate).to_string();
    let display = d.unwrap_or(&current_modes[0].display).to_string();
    let tmp = format!("{}x{}_{}",width,height,rate);
    // default test timeout is 10 seconds.
    let name = match n {
        Some(nm) => String::from(nm),
        None => {
            tmp
        }
    };
    let i_mode = InputMode {
        width,
        height,
        rate,
        display: String::from(&display),
        name: name.clone()
    };
    let mut d_vec: Vec<String> = Vec::with_capacity(1);
    d_vec.push(display.clone());
    // compute CVT timings and delete xrandr mode concurrently; wait for deletion before adding to xrandr
    //let del_handle = thread::spawn(move || delete_mode_xrandr(&name, Some(d_vec), verbose));
    let cvt_handle = thread::spawn(move || gen_cvt_mode(&i_mode, verbose));
    let fallback_cvt_handle = thread::spawn(move || gen_cvt_mode(&current_modes[0], verbose));
    //let _ = del_handle.join().unwrap();
    let cvt = cvt_handle.join().unwrap();
    let fallback_cvt = fallback_cvt_handle.join().unwrap();
    new_mode(&cvt, &display, verbose)?;
    if test {
        test_mode(&cvt, &fallback_cvt, &display, t, verbose)?;
    }
    if save {
        fileio::save_mode(&cvt,f,verbose)?
    }
    Ok(())
}


pub fn apply_mode(n: &str, d: &str, t: Option<&str>, test: bool, persist: bool, verbose: bool) -> Result<(), io::Error> {
    println!("Applying mode {} to display {}.",n,d);
    let mode = fileio::get_mode(n, None, verbose).unwrap();
    if test {
        let default_modes = get_default_modes(verbose)?;
        let default_mode = gen_cvt_mode(&default_modes[0],verbose);
        test_mode(&mode, &default_mode, d, t, verbose)?;
        println!("Keep the mode you just tested? y/n");
        let mut input = String::new();
        while !(input.contains("y") || input.contains("n")) {
            let _ = io::stdin().read_line(&mut input);
            if input.contains("n") {
                return Ok(());
            }
        }
    }
    switch_mode(n, d, verbose)?;
    if persist {
        fileio::save_mode_persistent(&mode, verbose)?;
    }
    Ok(())
}


fn test_mode(mode: &CvtMode, default_mode: &CvtMode, display: &str, t: Option<&str>, verbose: bool) -> Result<(), io::Error> {
    let name = &mode.get_name();
    let default_name = &default_mode.get_name();
    let timeout: u64 = match t {
        Some(time) => {
            let tmp = match time.parse() {
                Ok(kk) => kk,
                Err(_) => {
                    eprintln!("Error: timeout must be an integer greater than zero. Using default timeout of 10 seconds.");
                    10 // just default to 10 secs if invalid timeout provided rather than returning an error
                }
            };
            if tmp > 0 {
                tmp
            } else {
                10 // default to 10 secs if none given
            }
        }
        None => 10
    };
    let delay = time::Duration::from_secs(timeout);
    if verbose {
        println!("Testing mode {} on display {} for {} secs.", name, display, timeout);
        thread::sleep(time::Duration::from_secs(1));
    }
    if verbose {
        let _ = thread::spawn(move || util::print_countdown(timeout)); // this should maybe print regardless of verbose option, idk
    }
    let handle = thread::spawn(move || thread::sleep(delay));
    switch_mode(name, display, verbose)?;
    handle.join().expect("Timer thread had an error.");
    if verbose {
        println!("Reverting to mode {} on display {}.", default_name, display);
    }
    switch_mode(default_name, display, verbose)?;
    Ok(())
}


fn gen_cvt_mode(input: &InputMode, verbose: bool) -> CvtMode {
    if verbose {
        println!("Generating coordinated video timings for mode {}",input.name);
    }
    let mut cmd = process::Command::new("cvt");
    cmd.arg(&input.width).arg(&input.height).arg(&input.rate);
    let output = cmd.output().unwrap();
    let out = str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<_> = out.split('"').collect();
    let mut t: Vec<_> = lines[2][2..lines[2].len()-1].split(" ").collect();
    let mut i=0;
    while i < t.len() {
        if t[i] == "" || t[i] == "\t" {
            t.remove(i);
        } else {
            i += 1;
        }
    }
    let tmp = CvtMode {
        name: input.name.to_owned(),
        clock: String::from(t[0]),
        h_disp: String::from(t[1]),
        h_sync_start: String::from(t[2]),
        h_sync_end: String::from(t[3]),
        h_total: String::from(t[4]),
        v_disp: String::from(t[5]),
        v_sync_start: String::from(t[6]),
        v_sync_end: String::from(t[7]),
        v_total: String::from(t[8]),
        flags: format!("{} {}",t[9],t[10]),
    };
    if verbose {
        println!("{:?}",tmp);
    }
    tmp
}


// Retrieves modes which are currently in use
fn get_current_modes(verbose: bool) -> Result<Vec<InputMode>, Error> {
    if verbose {
        println!("Retrieving current display configuration.");
    }
    let re = Regex::new(r"(\S+)\s+connected.*\n[[a-zA-Z0-9\.]*\n]*\s*([0-9]+)x([0-9]+)\s*([0-9]+\.[0-9]+)\*").unwrap();
    util::get_modes_helper(&re, verbose)
}

// Retrieves the default modes for each display
fn get_default_modes(verbose: bool) -> Result<Vec<InputMode>, Error> {
    if verbose {
        println!("Retrieving current display configuration.");
    }
    let re = Regex::new(r"(\S+)\s+connected.*\n[[a-zA-Z0-9\.]*\n]*\s*([0-9]+)x([0-9]+)\s*([0-9]+\.[0-9]+)[\*]?\+").unwrap();
    util::get_modes_helper(&re, verbose)
}


fn switch_mode(name: &str, display: &str, verbose: bool) -> Result<(), io::Error> {
    let mut cmd = process::Command::new("xrandr");
    cmd.arg("--output").arg(&display).arg("--mode").arg(name);
    if verbose {
        println!("Applying mode {} to display {}",name,&display);
    }
    cmd.output()?;
    if verbose {
        println!("Successfully applied mode {} to display {}",name, &display);
    }
    Ok(())
}

// Adds the newly created mode to xrandr
fn new_mode(mode: &CvtMode, display: &str, verbose: bool) -> Result<(), io::Error> {
    let mut cmd = process::Command::new("xrandr");
    cmd.arg("--newmode")
        .arg(&mode.name)
        .arg(&mode.clock)
        .arg(&mode.h_disp)
        .arg(&mode.h_sync_start)
        .arg(&mode.h_sync_end)
        .arg(&mode.h_total)
        .arg(&mode.v_disp)
        .arg(&mode.v_sync_start)
        .arg(&mode.v_sync_end)
        .arg(&mode.v_total)
        .arg(&mode.flags);
    if verbose {
        println!("Creating xrandr mode {}",&mode.name);
    }
    cmd.output()?;
    if verbose {
        println!("Adding mode {} for display {}.",&mode.name,display);
    }
    cmd = process::Command::new("xrandr");
    cmd.arg("--addmode").arg(display).arg(&mode.name);
    cmd.output()?;
    Ok(())
}
