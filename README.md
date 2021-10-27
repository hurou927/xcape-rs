# xcape-rs

![Build](https://github.com/hurou927/xcape-rs/workflows/test/badge.svg)

Rust-powered XCAPE

xcape: https://github.com/alols/xcape

> xcape allows you to use a modifier key as another key when pressed and released on its own. Note that it is slightly slower than pressing the original key, because the pressed event does not occur until the key is released.


## Usage

```sh
$ cargo run -- -h

implement xcape  Rust

USAGE:
    xcape-rs [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug      debug flag
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --expression <map>...    format: code=code|code|code
    -t, --timeout <timeout>      timeout(sec).

```


### Sample

`xcape -e '64=38' # alt=space`

## Todo

- get Keysym from String

## Links

- crate.io: https://crates.io/crates/xcape-rs

