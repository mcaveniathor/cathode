use std::{env,fs,process,path,str,thread,time};
use std::io::Error;
use std::result::Result;
use regex::Regex;
use crate::mode::InputMode;


pub fn filename_or_default(f: Option<&str>,verbose: bool) -> Result<path::PathBuf, Error> {
    let buf = match f {
        Some(n) => {
            if verbose {
                println!("Using provided filename: {}.",n);
            }
            let tmp = path::PathBuf::from(n);
            if !tmp.parent().unwrap().is_dir() {
                if verbose {
                    println!("Directory {} or one of its parents does not exist. Creating them", &tmp.to_str().unwrap());
                }
                fs::create_dir_all(&tmp.parent().unwrap())?;
            }
            tmp
        },
        None => {
            let mut tmp = match env::var("XDG_CONFIG_HOME") {
                Ok(dir) => {
                    if verbose {
                        println!("$XDG_CONFIG_HOME is defined; using $XDG_CONFIG_HOME/cathode/modes.yml.");
                    }
                    path::PathBuf::from(dir)
                }
                Err(_) => {
                    if verbose {
                        println!("No filename provided and $XDG_CONFIG_HOME is not set; using $HOME/.config/cathode/modes.yml");
                    }
                    let mut p = path::PathBuf::from(env::var("HOME").unwrap());
                    p.push(".config");
                    p
                }
            };
            tmp.push("cathode");
            if !tmp.is_dir() {
                if verbose {
                    println!("Directory {} or one of its parents does not exist. Creating them.", &tmp.to_str().unwrap());
                }
                fs::create_dir_all(&tmp)?;
            }
            tmp.push("modes.yml");
            tmp
        }
    };
    Ok(buf)
}



pub fn get_modes_helper(re: &Regex, verbose: bool) -> Result<Vec<InputMode>, Error> {

    let mut modes: Vec<InputMode> = Vec::new();
     if cfg!(target_os = "linux") {
        let mut cmd = process::Command::new("xrandr");
        cmd.arg("--current");
        let dispoutput = cmd.output()?;
        let dispout = str::from_utf8(&dispoutput.stdout).unwrap();
        // capture groups: display name, default width, height, refresh rate for active displays
        for cap in re.captures_iter(dispout) {
            let width = cap[2].to_string();
            let height = cap[3].to_string();
            let rate = cap[4].to_string();
            let display = cap[1].to_string();
            let name = format!("{}x{}",width,height);
            if verbose {
                println!("Found mode {} for display {}: width: {}, height: {}, refresh rate: {}", name, display, width, height, rate);
            }
            let mode = InputMode::new(&width, &height, &rate, &display, &name);
            modes.push(mode);
        }
    }
    Ok(modes)
}


pub fn print_countdown(timeout: u64) {
    for i in 0..timeout {
        println!("Reverting in {} secs",timeout-i);
        thread::sleep(time::Duration::from_secs(1));
    }
}
