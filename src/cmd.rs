use std::path::Path;
use std::string::String;

use anyhow::Result;

use crate::display::{self, SizeUnit};
use crate::rules::Rules;

pub struct Command<'a> {
    rules: Rules<'a>,
    current_dir: &'a Path,
}

impl<'a> Command<'a> {
    pub fn new(rules: Rules<'a>, current_dir: &'a Path) -> Command<'a> {
        Command { rules, current_dir }
    }

    pub fn add_rules(&mut self, rules: Vec<&String>) -> Result<()> {
        self.rules.add(self.prefix_workdir(rules)?)
    }

    pub fn remove_rules(&mut self, rules: Vec<&String>) -> Result<()> {
        self.rules.remove(self.prefix_workdir(rules)?)
    }

    pub fn list(&self) {
        let patterns = self.rules.get();
        let total: u64 = patterns.iter().map(|p| p.get_size().unwrap_or(0)).sum();

        display::format_patterns(patterns);

        println!("----");
        println!("{}\ttotal to remove", SizeUnit::new(total));
    }

    fn prefix_workdir(&self, rules: Vec<&String>) -> Result<Vec<String>> {
        let mut paths: Vec<String> = Vec::new();
        for r in rules {
            if let Some(path) = self.current_dir.join(r).to_str() {
                paths.push(path.to_owned())
            }
        }
        Ok(paths)
    }
}
