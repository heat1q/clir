use std::io;
use std::path::Path;
use std::string::String;

use super::super::{config, fs};

pub struct Command<'a> {
    cfg: config::Config,
    current_dir: &'a Path,
}

impl<'a> Command<'a> {
    pub fn new(cfg: config::Config, current_dir: &'a Path) -> Command<'a> {
        Command { cfg, current_dir }
    }

    pub fn add_rules(&mut self, rules: Vec<&str>) -> io::Result<()> {
        self.cfg.rules.add(self.get_paths(rules)?)
    }

    pub fn remove_rules(&mut self, rules: Vec<&str>) -> io::Result<()> {
        self.cfg.rules.remove(self.get_paths(rules)?)
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

    fn get_paths(&self, rules: Vec<&str>) -> io::Result<Vec<String>> {
        let mut paths: Vec<String> = Vec::new();
        for r in rules {
            if let Some(path) = self.current_dir.join(r).canonicalize()?.to_str() {
                paths.push(path.to_owned())
            }
        }
        Ok(paths)
    }
}

fn to_humanreadable(size: u64) -> String {
    if size == 0 {
        return "".to_owned();
    }
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
