use clap::{App, Arg};

use super::context::Context;

fn parse_map(map: &str) -> (u8, Vec<u8>) {
    let v: Vec<&str> = map.split('=').collect();
    let key = v[0].parse::<u8>().unwrap();
    let vals: Vec<u8> = v[1].split('|').map(|m| m.parse::<u8>().unwrap()).collect();
    (key, vals)
}

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
                .multiple(true),
        )
        .arg(
            Arg::with_name("debug")
                .help("debug flag")
                .short("d")
                .long("debug"),
        )
        .get_matches();

    if let Some(maps) = app.values_of("map") {
        for map in maps {
            let v: Vec<&str> = map.split('=').collect();
            println!("map: {:?} {:?}", map, v);
        }
    }

    Context::new(app.is_present("debug"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_map_success() {
        assert_eq!(parse_map("16=32"), (16, vec![32]));
        assert_eq!(parse_map(" 16 = 32 "), (16, vec![32]));
        assert_eq!(parse_map("16=32|53|21"), (16, vec![32, 53, 21]));
    }
}
