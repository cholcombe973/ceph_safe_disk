use std::env;
use std::process;

use ceph_safe_disk::diag::{DiagMap, Format, Status};
use ceph_safe_disk::exec::check_user;
use ceph_safe_disk::exit::ExitStatus;

pub static NAME: &'static str = "ceph-safe-disk";

use getopts::Options;

fn print_help(opts: &Options) {
    println!("{0}", opts.usage("Usage: ceph-safe-disk [OPTION]"));
    println!(
        "Exit statuses:
    0: Safe to remove an OSD
    1: Not safe to remove an OSD
    2: General error"
    );
}

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();
    let mut options = Options::new();

    options.optflag("h", "help", "Print help information");
    options.optflag(
        "q",
        "quick",
        "Give a quick, non-exhaustive status of removable OSDs",
    );
    options.optflag(
        "e",
        "exhaustive",
        "Give an exhaustive status of removable OSDs",
    );
    options.optopt(
        "f",
        "format",
        "Format output where the options are: pretty, json",
        "FORMAT",
    );

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            println!("{}: {}", NAME, err.to_string());
            process::exit(ExitStatus::Err as i32);
        }
    };

    if matches.opt_present("h") {
        print_help(&options);
    } else {
        if let Err(user_err) = check_user() {
            println!("{}: {}", NAME, user_err.to_string());
            return ExitStatus::Err as i32;
        };
        let mut format = Format::Pretty;
        if matches.opt_present("f") {
            if let Some(format_arg) = matches.opt_str("f") {
                if let "json" = format_arg.as_ref() {
                    format = Format::Json
                }
            }
        }
        match DiagMap::new() {
            Ok(diag_map) => {
                if matches.opt_present("q") {
                    if diag_map.quick_diag(format) {
                        return ExitStatus::SafeRm as i32;
                    } else {
                        return ExitStatus::NonSafeRm as i32;
                    }
                } else if matches.opt_present("e") {
                    match diag_map.exhaustive_diag(format) {
                        Status::Safe => return ExitStatus::SafeRm as i32,
                        Status::NonSafe => return ExitStatus::NonSafeRm as i32,
                        _ => return ExitStatus::Err as i32,
                    }
                }
            }
            Err(err) => {
                print!("{}: {}", NAME, err.to_string());
                return ExitStatus::Err as i32;
            }
        }
    }
    ExitStatus::Err as i32
}

fn main() {
    match run() {
        error @ 0...2 => process::exit(error),
        _ => process::exit(ExitStatus::Err as i32),
    };
}
