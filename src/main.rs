// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's full 'builder pattern' style of creating arguments which is
// more verbose, but allows easier editing, and at times more advanced options, or the possibility
// to generate arguments dynamically.
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate toml;
extern crate regex;

use std::fs::File;
use std::io::prelude::*;
use std::process::exit;
use std::io;

use clap::{Arg, App};
use toml::Value;
use regex::Regex;

error_chain!{}

fn _regex_query(needle: &str, expr: &str) -> i32 {
    // Check if this is a sed-style query matching `/expr/repl/`.
    let sed_expr = Regex::new("/(.*)/(.*)/").unwrap();
    if let Some(sed_captures) = sed_expr.captures(expr) {
        println!("{}, {}",
                 sed_captures.get(1).unwrap().as_str(),
                 sed_captures.get(2).unwrap().as_str());
    }

    // Otherwise, just filter captures.
    let expr = Regex::new(expr)
        .expect(&format!("Couldn't build a regular expression from {}", expr));

    println!("{}", needle);
    for capture in expr.captures_iter(needle) {
        println!("{}", capture.get(0).unwrap().as_str());
    }

    0
}

fn main() {
    let matches = App::new(crate_name!())
                          .version(crate_version!())
                          .author(crate_authors!())
                          .about(crate_description!())
                          .arg(Arg::with_name("file")
                               .short("f")
                               .long("file")
                               .value_name("TOML_FILE")
                               .help("TOML file to read")
                               .takes_value(true))
                          .arg(Arg::with_name("regex")
                               .short("r")
                               .long("regex")
                               .value_name("REGULAR_EXPRESSION")
                               .help("Regular expression to apply.  Use `/query/replacement/` to change values inline, or just `expr` to filter your query.")
                               .takes_value(true))
                          .arg(Arg::with_name("PATTERN")
                               .help("Field to read from the TOML file")
                               .required(true)
                               .index(1))
                          // .arg(Arg::with_name("url")
                          //      .short("u")
                          //      .long("url")
                          //      .value_name("URL_PATH")
                          //      .help("TOML file URL to read")
                          //      .takes_value(true))
                          .get_matches();

    let toml_file = match (matches.value_of("file"), matches.value_of("url")) {
        (Some(f), None) => {
            load_toml_from_file(f).unwrap()
        }
        (None, Some(_u)) => {
            unimplemented!()
        }
        (None, None) => {
            eprintln!("Must specify URL or File to load!");
            ::std::process::exit(-1)
        }
        (Some(_), Some(_)) => {
            eprintln!("Cannot specify URL and File!");
            ::std::process::exit(-1)
        }
    };

    let pattern = matches
        .value_of("PATTERN").unwrap();

    let x = pattern
        .split('.')
        .fold(Some(&toml_file), |acc, key| {
            match acc {
                Some(a) => a.get(key),
                None => None
            }
        });

    let re = matches.value_of("regex");

    exit(match x {
        Some(needle) => {
            let needle = format!("{}", needle);
            let needle = needle.trim_matches('"');

            match re {
                Some(expr) => _regex_query(needle, expr),
                None => {
                    println!("{}", needle);
                    0
                }
            }
        }
        None => {
            writeln!(io::stderr(), "{} not found!", pattern).unwrap();
            -1
        }

    });
}

fn load_toml_from_file(file_name: &str) -> Result<Value> {
    let mut file = File::open(file_name)
        .chain_err(|| format!("Failed to open file: {:?}", &file_name))?;
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    toml::from_str(&contents).chain_err(|| "Deserialize error")
}
