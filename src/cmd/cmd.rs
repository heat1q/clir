use std::io::Result;
use std::string::String;

use super::super::{config, fs};

pub struct Command {
    cfg: config::Config,
}

impl Command {
    pub fn new(cfg: config::Config) -> Command {
        Command { cfg }
    }

    pub fn add_rules(&mut self, rules: &Vec<&str>) -> Result<()> {
        self.cfg.rules.add(rules)
    }

    pub fn remove_rules(&mut self, rules: &Vec<&str>) -> Result<()> {
        self.cfg.rules.remove(rules)
    }

    pub fn list(&self) {
        let rules = self.cfg.rules.get();
        let mut total: u64 = 0;
        for r in rules {
            let size = match fs::get_size(&r) {
                Ok(a) => a,
                Err(_) => continue,
            };

            total += size;

            println!("{}\t{}", to_humanreadable(size), r);
        }

        println!("----");
        println!("{}\ttotal to remove", to_humanreadable(total));
    }
}

fn to_humanreadable(size: u64) -> String {
    let exp: u64 = 1000;
    let mut i = 0;
    let mut res = size;
    while res > 0 {
        res /= exp;
        i += 1;
    }

    let s = size / (exp.pow(i - 1));
    match i - 1 {
        1 => s.to_string() + "K",
        2 => s.to_string() + "M",
        3 => s.to_string() + "G",
        4 => s.to_string() + "T",
        _ => s.to_string(),
    }
}
