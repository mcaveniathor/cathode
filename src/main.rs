#[macro_use]
extern crate clap;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate yaml_rust;
use std::io::Error;
use std::result::Result;

mod fileio;
mod mode;
mod util;

fn main() -> Result<(), Error> {
    let matches = clap_app!(cathode =>
                            (version: "0.1.0")
                            (author: "Thor McAvenia <mcaveniathor@gmail.com>")
                            (@arg verbose: -v --verbose "Enable verbose output for all subcommands.")
                            (@arg import: -i --import "Load modes from $HOME/.config/cathode/modes.yml or the file specified by the FILENAME parameter")
                            (@arg filename: -f --filename [filename] "Specify a modes file to load")
                            (@subcommand add =>
                                (about: "create a new mode.")
                                (@arg width: -w --width [width] "width in pixels. defaults to the currently active value")
                                (@arg height: -h --height [height] "display height in pixels defaults to the currently active value.")
                                (@arg rate: -r --rate [rate] "refresh rate in hz. defaults to the currently active value.")
                                (@arg display: -d --display [display] "display for which to add the mode. defaults to the first connected display.")
                                (@arg name: -n --name [name] "the name of the mode. defaults to <width>x<height>_<rate>")
                                (@arg timeout: -t --timeout [timeout] "Specify a timeout duration in seconds. Implies --test.")
                                (@arg test: --test "Apply this mode temporarily to see if it works (useful for monitor overclocking). Reverts to the default mode after 10 seconds or TIMEOUT if --timeout is used.")
                                (@arg nosave: --nosave "Do not write this mode to file.")
                                (@arg verbose: -v --verbose "Enable verbose output for add subcommand.")
                            )
                            (@subcommand apply =>
                             (about: "Apply a display mode to a display.")
                             (@arg name: -n --name <name> "Name of the mode to be applied.")
                             (@arg display: -d --display <display> "Display to which the mode should be applied.")
                             (@arg test: --test "Apply this mode temporarily to see if it works (useful for monitor overclocking). Reverts to the default mode after 10 seconds or TIMEOUT if --timeout is used.")
                             (@arg timeout: -t --timeout [timeout] "Specify a timeout duration. Implies --test.")
                             (@arg persist: -p --persist "Automatically apply this mode when you log in to this user. This places xrandr commands in $HOME/.xprofile.")
                             (@arg verbose: -v --verbose "Enable verbose output for apply subcommand.")
                            )
                           ).get_matches();
    // TODO: automatic OC
    let v = matches.is_present("verbose");
    let filename = matches.value_of("filename");
    if matches.is_present("import") {
        fileio::import_all_modes(filename,v)?;
    }
    if let Some(addmatches) = matches.subcommand_matches("add") {
        let verbose = v || addmatches.is_present("verbose");
        let width = addmatches.value_of("width");
        let height = addmatches.value_of("height");
        let rate = addmatches.value_of("rate");
        let name = addmatches.value_of("name");
        let display = addmatches.value_of("display");
        let test = addmatches.is_present("test") || addmatches.is_present("timeout");
        let timeout = addmatches.value_of("timeout");
        let save = !addmatches.is_present("nosave");
        return mode::add_mode(width, height, rate, display, name, timeout, filename, test, save, verbose)
    }
    if let Some(applymatches) = matches.subcommand_matches("apply") {
        let verbose = v || applymatches.is_present("verbose");
        let name = applymatches.value_of("name").unwrap(); // required; unwrap rather than error check
        let test = applymatches.is_present("test") || applymatches.is_present("timeout");
        let timeout = applymatches.value_of("test");
        let display = applymatches.value_of("display").unwrap(); // required; unwrap rather than error check
        let persist = applymatches.is_present("persist");
        return mode::apply_mode(name,display,timeout,test,persist,verbose)
    }
    Ok(())
}

