
use clap::{App, Arg};

use super::context::Context; 

pub fn parse() -> Context {
    let app = App::new("xcape-rs")
        .version("1.0")
        .about("implement xcape with Rust")
        .arg(
            Arg::with_name("map")
            .help("format: {keycode}={keycode}")
            .takes_value(true)
            .short("e")
            .long("expression")
            .multiple(true)
        )
        .arg(
            Arg::with_name("debug")
            .help("debug flag")
            .short("d")
            .long("debug")
            )
        .get_matches();

    if let Some(in_v) = app.values_of("map") {
        for in_file in in_v {
            println!("map: {}", in_file);
        }
    } 


    Context::new(app.is_present("debug"))
}
