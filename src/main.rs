mod cli;
mod copy;
mod utils;

use cli::parse_args;
use copy::copy_sources;

fn main() {
    let args = parse_args();

    if let Err(e) = copy_sources(&args.sources, &args.destination, &args.options) {
        let prog = std::env::args().next().unwrap_or_else(|| "cprust".into());
        eprintln!("{}: {}", prog, e);
        std::process::exit(1);
    }
}
