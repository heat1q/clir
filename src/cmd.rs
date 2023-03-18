use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::string::String;
use std::time;

use anyhow::{Ok, Result};

use crate::display;
use crate::path::PathTree;
use crate::rules::{Pattern, Rules};

pub(crate) struct Command<'a> {
    rules: Rules<'a>,
    workdir: &'a Path,
    verbose_mode: bool,
    absolute_path: bool,
}

impl<'a> Command<'a> {
    pub(crate) fn new(
        rules: Rules<'a>,
        workdir: &'a Path,
        verbose_mode: bool,
        absolute_path: bool,
    ) -> Command<'a> {
        Command {
            rules,
            workdir,
            verbose_mode,
            absolute_path,
        }
    }

    pub(crate) fn add_rules(&mut self, rules: Vec<&String>) -> Result<()> {
        self.rules.add(self.prefix_workdir(rules)?)
    }

    pub(crate) fn remove_rules(&mut self, rules: Vec<&String>) -> Result<()> {
        self.rules.remove(self.prefix_workdir(rules)?)
    }

    pub(crate) fn list(&self) -> Result<Vec<Pattern>> {
        let mut path_tree = PathTree::new();
        let patterns = self.rules.expand_patterns(&mut path_tree);
        display::format_patterns(self.workdir, &path_tree, &patterns, self.absolute_path)?;
        Ok(patterns)
    }

    pub(crate) fn clean(&self) -> Result<()> {
        let patterns = self.list()?;
        print!("\nClean all selected paths? [(Y)es/(N)o]: ");
        stdout().lock().flush()?;

        let mut confirm = "".to_owned();
        stdin().read_line(&mut confirm)?;

        if confirm.starts_with('y') || confirm.starts_with('Y') {
            let start = time::Instant::now();
            self.rules.clean(&patterns, self.verbose_mode)?;
            let elapsed = start.elapsed().as_millis();
            println!("Finished in {:.2}s", (elapsed as f64) / 1000.);
        } else {
            println!("Aborting...");
        }

        Ok(())
    }

    fn prefix_workdir(&self, rules: Vec<&String>) -> Result<Vec<String>> {
        let mut paths: Vec<String> = Vec::new();
        for r in rules {
            if let Some(path) = self.workdir.join(r).to_str() {
                paths.push(path.to_owned())
            }
        }
        Ok(paths)
    }
}
