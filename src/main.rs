#[macro_use]
extern crate log;
extern crate env_logger as logger;

use std::env;
use std::sync::Arc;

mod processor;
use processor::args;
use processor::runner;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let ctx = args::parse()?;
    if ctx.is_debug_mode() {
        env::set_var("RUST_LOG", "debug");
    }

    logger::init();
    debug!("ctx {:?}", ctx);

    runner::run(&ctx)?;

    Ok(())
}
