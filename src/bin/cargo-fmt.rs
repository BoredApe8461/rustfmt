// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Inspired by Paul Woolcock's cargo-fmt (https://github.com/pwoolcoc/cargo-fmt/)

#![cfg(not(test))]
#![cfg(feature="cargo-fmt")]
#![deny(warnings)]

extern crate getopts;
extern crate rustc_serialize;

use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::str;

use getopts::Options;
use rustc_serialize::json::Json;

fn main() {
    let exit_status = execute();
    std::io::stdout().flush().unwrap();
    std::process::exit(exit_status);
}

fn execute() -> i32 {
    let success = 0;
    let failure = 1;

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "show this message");
    opts.optflag("q", "quiet", "no output printed to stdout");
    opts.optflag("v", "verbose", "use verbose output");

    let matches = match opts.parse(env::args().skip(1).take_while(|a| a != "--")) {
        Ok(m) => m,
        Err(e) => {
            print_usage(&opts, &e.to_string());
            return failure;
        }
    };

    let verbosity = match (matches.opt_present("v"), matches.opt_present("q")) {
        (false, false) => Verbosity::Normal,
        (false, true) => Verbosity::Quiet,
        (true, false) => Verbosity::Verbose,
        (true, true) => {
            print_usage(&opts, "quiet mode and verbose mode are not compatible");
            return failure;
        }
    };

    if matches.opt_present("h") {
        print_usage(&opts, "");
        return success;
    }

    match format_crate(verbosity) {
        Err(e) => {
            print_usage(&opts, &e.to_string());
            failure
        }
        Ok(status) => {
            if status.success() {
                success
            } else {
                status.code().unwrap_or(failure)
            }
        }
    }
}

fn print_usage(opts: &Options, reason: &str) {
    let msg = format!("{}\nusage: cargo fmt [options]", reason);
    println!("{}\nThis utility formats all bin and lib files of the current crate using rustfmt. \
              Arguments after `--` are passed to rustfmt.",
             opts.usage(&msg));
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verbosity {
    Verbose,
    Normal,
    Quiet,
}

fn format_crate(verbosity: Verbosity) -> Result<ExitStatus, std::io::Error> {
    let targets = try!(get_targets());

    // Currently only bin and lib files get formatted
    let files: Vec<_> = targets.into_iter()
                               .filter(|t| t.kind.is_lib() | t.kind.is_bin())
                               .inspect(|t| {
                                   if verbosity == Verbosity::Verbose {
                                       println!("[{:?}] {:?}", t.kind, t.path)
                                   }
                               })
                               .map(|t| t.path)
                               .collect();

    format_files(&files, &get_fmt_args(), verbosity)
}

fn get_fmt_args() -> Vec<String> {
    // All arguments after -- are passed to rustfmt
    env::args().skip_while(|a| a != "--").skip(1).collect()
}

#[derive(Debug)]
enum TargetKind {
    Lib, // dylib, staticlib, lib
    Bin, // bin
    Other, // test, plugin,...
}

impl TargetKind {
    fn is_lib(&self) -> bool {
        match self {
            &TargetKind::Lib => true,
            _ => false,
        }
    }

    fn is_bin(&self) -> bool {
        match self {
            &TargetKind::Bin => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Target {
    path: PathBuf,
    kind: TargetKind,
}

// Returns a vector of all compile targets of a crate
fn get_targets() -> Result<Vec<Target>, std::io::Error> {
    let mut targets: Vec<Target> = vec![];
    let output = try!(Command::new("cargo").arg("read-manifest").output());
    if output.status.success() {
        // None of the unwraps should fail if output of `cargo read-manifest` is correct
        let data = &String::from_utf8(output.stdout).unwrap();
        let json = Json::from_str(data).unwrap();
        let jtargets = json.find("targets").unwrap().as_array().unwrap();
        for jtarget in jtargets {
            targets.push(target_from_json(jtarget));
        }

        Ok(targets)
    } else {
        // This happens when cargo-fmt is not used inside a crate
        Err(std::io::Error::new(std::io::ErrorKind::NotFound,
                                str::from_utf8(&output.stderr).unwrap()))
    }
}

fn target_from_json(jtarget: &Json) -> Target {
    let jtarget = jtarget.as_object().unwrap();
    let path = PathBuf::from(jtarget.get("src_path").unwrap().as_string().unwrap());
    let kinds = jtarget.get("kind").unwrap().as_array().unwrap();
    let kind = match kinds[0].as_string().unwrap() {
        "bin" => TargetKind::Bin,
        "lib" | "dylib" | "staticlib" => TargetKind::Lib,
        _ => TargetKind::Other,
    };

    Target {
        path: path,
        kind: kind,
    }
}

fn format_files(files: &Vec<PathBuf>,
                fmt_args: &Vec<String>,
                verbosity: Verbosity)
                -> Result<ExitStatus, std::io::Error> {
    let stdout = if verbosity == Verbosity::Quiet {
        std::process::Stdio::null()
    } else {
        std::process::Stdio::inherit()
    };
    if verbosity == Verbosity::Verbose {
        print!("rustfmt");
        for a in fmt_args.iter() {
            print!(" {}", a);
        }
        for f in files.iter() {
            print!(" {}", f.display());
        }
        println!("");
    }
    let mut command = try!(Command::new("rustfmt")
        .stdout(stdout)
        .args(files)
        .args(fmt_args)
        .spawn());
    command.wait()
}
