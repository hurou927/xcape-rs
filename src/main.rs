
mod processor;
use processor::args;
use processor::runner;

fn main() {

    args::parse();

    runner::run();
}
