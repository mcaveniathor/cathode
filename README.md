# cathode
A CLI program which facilitates monitor overclocking and resolution adjustment under XOrg.
Depends on cvt to generate monitor timings based on provided resolution and xrandr to apply changes.

USAGE:
    cathode [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -i, --import     Load modes from $HOME/.config/cathode/modes.yml or the file specified by the FILENAME parameter
    -V, --version    Prints version information
    -v, --verbose    Enable verbose output for all subcommands.

OPTIONS:
    -f, --filename <filename>    Specify a modes file to load

SUBCOMMANDS:
    add      create a new mode.
    apply    Apply a display mode to a display.
    help     Prints this message or the help of the given subcommand(s)
