extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::env;

  //naval_fate.py ship <name> move <x> <y> [--speed=<kn>]
  //naval_fate.py ship shoot <x> <y>
  //naval_fate.py mine (set|remove) <x> <y> [--moored | --drifting]
  //naval_fate.py (-h | --help)
  //naval_fate.py --version


static USAGE: &'static str = "
bootstrap.

Usage:
  bootstrap env (new|list) <name>
  bootstrap ((-h | --help))
  bootstrap --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_name: Vec<String>,
    cmd_env: bool
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
}

