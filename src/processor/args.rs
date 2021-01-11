use clap::{App, Arg};
use std::collections::HashMap;

use super::context::Context;
use super::error::XcapeError;
use std::error::Error;

fn parse_map(map: &str) -> Result<(u8, Vec<u8>), Box<dyn Error>> {
    let v: Vec<&str> = map.split('=').map(|m| m.trim()).collect();

    if v.len() != 2 {
        return Err(XcapeError::InvalidExpressionArg {
            map: map.to_string(),
            reason: "use `=`.".to_string(),
        })?;
    }

    let key = match v[0].parse::<u8>() {
        Ok(x) => x,
        Err(_) => {
            return Err(XcapeError::InvalidExpressionArg {
                map: map.to_string(),
                reason: "parserInt key error".to_string(),
            })?
        }
    };
    let vals: Vec<u8> = match v[1].split('|').map(|m| m.trim().parse::<u8>()).collect() {
        Ok(vs) => vs,
        Err(_) => {
            return Err(XcapeError::InvalidExpressionArg {
                map: map.to_string(),
                reason: "parseInt value error".to_string(),
            })?
        }
    };
    Ok((key, vals))
}

pub fn parse() -> Result<Context, Box<dyn Error>> {
    let app = App::new("xcape-rs")
        .version("1.0")
        .about("implement xcape with Rust")
        .arg(
            Arg::with_name("map")
                .help("format: code=code|code|code")
                .takes_value(true)
                .short("e")
                .long("expression")
                .multiple(true),
        )
        .arg(
            Arg::with_name("timeout")
                .help("timeout(sec).")
                .takes_value(true)
                .short("t")
                .long("timeout"),
        )
        .arg(
            Arg::with_name("debug")
                .help("debug flag")
                .short("d")
                .long("debug"),
        )
        .get_matches();

    let mut h: HashMap<u8, Vec<u8>> = HashMap::new();
    if let Some(maps) = app.values_of("map") {
        for map in maps {
            let p = parse_map(map)?;
            h.insert(p.0, p.1);
        }
    }

    let timeout_sec: Option<u64> = match app.value_of("timeout").map(|x| {
        x.parse::<u64>()
            .map_err(|_| XcapeError::InvalidArg("fail to parse timeout.".to_string()))
    }) {
        None => None,
        Some(res) => Some(res?),
    };

    let ctx = Context::new(app.is_present("debug"), timeout_sec, h);
    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_map_success() {
        assert_eq!(parse_map("16=32").unwrap(), (16, vec![32]));
        assert_eq!(parse_map(" 16 = 32 ").unwrap(), (16, vec![32]));
        assert_eq!(parse_map("16=32|53|21").unwrap(), (16, vec![32, 53, 21]));
    }

    #[test]
    fn parse_map_failure() {
        assert!(parse_map("256 = 32").is_err());
        assert!(parse_map("hoge = fuga").is_err());
        assert!(parse_map("256 = 32*32*52").is_err());
        assert!(parse_map("423").is_err());
    }
}
