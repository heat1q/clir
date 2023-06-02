use crate::cmd::Command;
use crate::rules::Rules;
use anyhow::{anyhow, Ok, Result};
use clap::{App, Arg};
use simple_logger::SimpleLogger;
use std::{env, path::Path};

mod cmd;
mod display;
mod path;
mod rules;

pub fn run() -> Result<()> {
    let current_dir = env::current_dir()?;
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
            Arg::new("config")
                .help("Path to alternative config file.")
                .short('c')
                .action(clap::ArgAction::Set)
                .default_value(".clir")
                .value_hint(clap::ValueHint::FilePath),
        )
        .arg(
            Arg::new("verbose")
                .help("Run in verbose mode")
                .short('v')
                .action(clap::ArgAction::Count),
        )
        .arg(
            Arg::new("absolute")
                .help("Display absolute paths")
                .short('a')
                .long("absolute")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("run")
                .help("Recursively clean all defined patterns")
                .short('r')
                .action(clap::ArgAction::SetTrue),
        );

    if let Err(err) = parse_args(&mut app, &current_dir) {
        app.print_help()?;
        return Err(err);
    }

    Ok(())
}

fn parse_args(app: &mut App, current_dir: &Path) -> Result<()> {
    let app = app.get_matches_mut();
    let verbosity_level = *app.get_one::<u8>("verbose").unwrap_or(&0);
    let absolute_path = *app.get_one::<bool>("absolute").unwrap_or(&false);
    let config_path = app.get_one::<String>("config").unwrap();

    setup_logger(verbosity_level);
    log::trace!("working dir: {}", current_dir.display());

    let rules = Rules::new(config_path.as_ref())?;
    let mut cmd = Command::new(rules, current_dir, absolute_path);

    if *app.get_one::<bool>("run").unwrap() {
        return cmd.clean();
    }

    match app.subcommand() {
        Some(("add", p)) => {
            let rules: Vec<&String> = p
                .get_many("pattern")
                .ok_or_else(|| anyhow!("invalid patterns for `add`"))?
                .collect();
            cmd.add_rules(rules)
        }
        Some(("remove", p)) => {
            let rules: Vec<&String> = p
                .get_many("pattern")
                .ok_or_else(|| anyhow!("invalid patterns for `remove`"))?
                .collect();
            cmd.remove_rules(rules)
        }
        _ => cmd.list().map(|_| ()),
    }
}

fn setup_logger(verbosity_level: u8) {
    let level_filter = match verbosity_level {
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Error,
    };
    SimpleLogger::new().with_level(level_filter).init().unwrap();
}
