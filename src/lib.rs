use crate::cmd::Command;
use crate::rules::Rules;
use anyhow::{anyhow, Ok, Result};
use clap::{App, Arg};
use std::env;

mod cmd;
mod display;
mod rules;

pub fn run() -> Result<()> {
    #[cfg(feature = "env_logger")]
    env_logger::init();

    let current_dir = env::current_dir()?;
    log::info!("working dir: {}", current_dir.display());

    let mut app = App::new("clir")
        .about("A command line cleaning utility.")
        .subcommand(
            App::new("add")
                .about("Add new path(s) or glob pattern(s)")
                .arg(
                Arg::new("pattern")
                    .help(
                        "One or more paths or patterns. Paths can either be relative or absolute.",
                    )
                    .multiple_values(true),
            ),
        )
        .subcommand(
            App::new("remove").about("Remove paths or patterns").arg(
                Arg::new("pattern")
                    .help(
                        "One or more paths or patterns. Paths can either be relative or absolute.",
                    )
                    .multiple_values(true),
            ),
        )
        .arg(
            Arg::new("run")
                .help("Recursively clean all defined patterns")
                .short('r')
                .action(clap::ArgAction::SetTrue),
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
    let app = app.get_matches_mut();
    if *app.get_one::<bool>("run").unwrap() {
        log::info!("run clean");
        return cmd.clean();
    }

    match app.subcommand() {
        Some(("add", p)) => {
            let rules: Vec<&String> = p
                .get_many("pattern")
                .ok_or_else(|| anyhow!("invalid patterns for `add`"))?
                .collect();
            cmd.add_rules(rules)?;
        }
        Some(("remove", p)) => {
            let rules: Vec<&String> = p
                .get_many("pattern")
                .ok_or_else(|| anyhow!("invalid patterns for `remove`"))?
                .collect();
            cmd.remove_rules(rules)?;
        }
        _ => cmd.list(),
    };

    Ok(())
}
