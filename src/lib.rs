use crate::cmd::Command;
use crate::rules::Rules;
use anyhow::{anyhow, Ok, Result};
use clap::{App, Arg};
use std::env;

mod cmd;
mod display;
mod rules;

pub fn run() -> Result<()> {
    let current_dir = env::current_dir()?;
    println!("working dir: {}", current_dir.display());

    let mut app = App::new("clir")
        .about("Does awesome things")
        .subcommand(
            App::new("add")
                .about("adds new rules")
                .arg(Arg::new("pattern").multiple_values(true)),
        )
        .subcommand(
            App::new("remove")
                .about("remove rules")
                .arg(Arg::new("pattern").multiple_values(true)),
        );

    let rules = Rules::new(".clir".as_ref())?;
    let cmd = Command::new(rules, &current_dir);

    if let Err(err) = parse_args(&mut app, cmd) {
        app.print_help()?;
        return Err(err);
    }

    Ok(())
}

fn parse_args(app: &mut App, mut cmd: Command) -> Result<()> {
    match app.get_matches_mut().subcommand() {
        Some(("add", p)) => {
            let rules: Vec<&str> = p
                .get_many("pattern")
                .ok_or(anyhow!("invalid patterns for `add`"))?
                .copied()
                .collect();
            cmd.add_rules(rules)?;
        }
        Some(("remove", p)) => {
            let rules: Vec<&str> = p
                .get_many("pattern")
                .ok_or(anyhow!("invalid patterns for `remove`"))?
                .copied()
                .collect();
            cmd.remove_rules(rules)?;
        }
        _ => cmd.list(),
    };

    Ok(())
}
