use anyhow::Result;
use clap::{App, Arg};
use std::env;

use crate::cmd::Command;
use crate::rules::Rules;

mod cmd;
mod rules;

pub fn run() -> Result<()> {
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

    let mut rules = Rules::new(".clir")?;
    let mut cmd = Command::new(&mut rules, current_dir);

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
