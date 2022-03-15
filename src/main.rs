use clap::{App, Arg};

mod cmd;
mod config;
mod fs;

use cmd::Command;
use config::Config;
use std::env;

fn main() {
    let current_dir = env::current_dir().unwrap();
    println!("working dir: {}", current_dir.display());
    let matches = App::new("clir")
        .about("Does awesome things")
        .subcommand(
            App::new("add")
                .about("adds a new rule")
                .arg(Arg::new("pattern").multiple_values(true)),
        )
        .subcommand(
            App::new("remove")
                .about("remove rules")
                .arg(Arg::new("pattern").multiple_values(true)),
        )
        .get_matches();

    let cfg = Config::new(".clir");
    let mut cmd = Command::new(cfg, current_dir.as_path());

    match matches.subcommand() {
        Some(("add", p)) => {
            let vals: Vec<&str> = p.values_of("pattern").unwrap().collect();
            println!("input: {:?}", vals);
            cmd.add_rules(vals).unwrap();
        }
        Some(("remove", p)) => {
            let vals: Vec<&str> = p.values_of("pattern").unwrap().collect();
            println!("input: {:?}", vals);
            cmd.remove_rules(vals).unwrap();
        }
        _ => cmd.list(),
    }
}
