extern crate clap;
extern crate d_mark;

use clap::{App, Arg};
use d_mark::Parser;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("input")
                .help("Sets the input file to use")
                .index(1),
        ).get_matches();
    let filename = matches.value_of("input").unwrap_or("-");

    // Read file
    let mut contents = String::new();
    if filename == "-" {
        io::stdin()
            .read_to_string(&mut contents)
            .expect("stdin not readable");
    } else {
        let mut file = File::open(filename).expect("file not found");
        file.read_to_string(&mut contents)
            .expect("file not readable");
    };

    // Parse
    let res = Parser::call(&contents);
    match res {
        Ok(parsed) => println!("{:#?}", parsed),
        Err(error) => println!("{}", error),
    };
}
