
extern crate env_logger as logger;

use std::sync::Arc;
use std::env;

mod processor;
use processor::args;
use processor::runner;

fn main() {

    let ctx = args::parse();
    
    if ctx.is_debug_mode() {
        env::set_var("RUST_LOG", "debug");
    }
    logger::init();

    runner::run(Arc::new(ctx));
}
