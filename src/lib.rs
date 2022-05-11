use clap::{App, Arg};
use std::{env, error::Error};

use crate::cmd::Command;
use crate::config::Config;

mod cmd;
mod config;
mod rules;

pub fn run() -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
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

    let cfg = Config::new(".clir")?;
    let mut cmd = Command::new(cfg, current_dir.as_path());

    match matches.subcommand() {
        Some(("add", p)) => {
            if let Some(vals) = p.values_of("pattern") {
                cmd.add_rules(vals.collect())?;
            }
        }
        Some(("remove", p)) => {
            if let Some(vals) = p.values_of("pattern") {
                cmd.remove_rules(vals.collect())?;
            }
        }
        _ => cmd.list(),
    }
    Ok(())
}
