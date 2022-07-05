use std::path::Path;
use std::string::String;

use anyhow::Result;

use crate::display::SizeUnit;
use crate::rules::Rules;

pub struct Command<P, V> {
    rules: Rules<P>,
    current_dir: V,
}

impl<P, V> Command<P, V>
where
    P: AsRef<Path>,
    V: AsRef<Path>,
{
    pub fn new(rules: Rules<P>, current_dir: V) -> Command<P, V> {
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
            if let Some(path) = self.current_dir.as_ref().join(r).to_str() {
                paths.push(path.to_owned())
            }
        }
        Ok(paths)
    }
}
