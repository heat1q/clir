use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::string::String;

use anyhow::{Ok, Result};

use crate::display;
use crate::rules::Rules;

pub struct Command<'a> {
    rules: Rules<'a>,
    workdir: &'a Path,
}

impl<'a> Command<'a> {
    pub fn new(rules: Rules<'a>, workdir: &'a Path) -> Command<'a> {
        Command { rules, workdir }
    }

    pub fn add_rules(&mut self, rules: Vec<&String>) -> Result<()> {
        self.rules.add(self.prefix_workdir(rules)?)
    }

    pub fn remove_rules(&mut self, rules: Vec<&String>) -> Result<()> {
        self.rules.remove(self.prefix_workdir(rules)?)
    }

    pub fn list(&self) {
        display::format_patterns(self.workdir, self.rules.get());
    }

    pub fn clean(&self) -> Result<()> {
        self.list();
        let mut confirm = "".to_owned();
        print!("Clean all selected paths? [(Y)es/(N)o]: ");
        stdout().lock().flush()?;
        stdin().read_line(&mut confirm)?;
        if confirm == "y" || confirm == "Y" {
            self.rules.clean()
        } else {
            println!("Aborting...");
            Ok(())
        }
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
