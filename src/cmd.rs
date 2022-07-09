use std::path::Path;
use std::string::String;

use anyhow::Result;

use crate::display::SizeUnit;
use crate::rules::Rules;

pub struct Command<'a> {
    rules: Rules<'a>,
    current_dir: &'a Path,
}

impl<'a> Command<'a> {
    pub fn new(rules: Rules<'a>, current_dir: &'a Path) -> Command<'a> {
        Command { rules, current_dir }
    }

    pub fn add_rules(&mut self, rules: Vec<&str>) -> Result<()> {
        self.rules.add(self.prefix_workdir(rules)?)
    }

    pub fn remove_rules(&mut self, rules: Vec<&str>) -> Result<()> {
        self.rules.remove(self.prefix_workdir(rules)?)
    }

    pub fn list(&self) {
        let patterns = self.rules.get();
        let mut total: u64 = 0;
        for pattern in patterns {
            if let Some(size) = pattern.get_size() {
                total += size;
                println!(
                    "{}\t{} ({} files, {} dirs)",
                    SizeUnit::new(size),
                    pattern,
                    pattern.num_files(),
                    pattern.num_dirs()
                );
            }
        }

        println!("----");
        println!("{}\ttotal to remove", SizeUnit::new(total));
    }

    fn prefix_workdir(&self, rules: Vec<&str>) -> Result<Vec<String>> {
        let mut paths: Vec<String> = Vec::new();
        for r in rules {
            if let Some(path) = self.current_dir.join(r).to_str() {
                paths.push(path.to_owned())
            }
        }
        Ok(paths)
    }
}
