use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Result, Write};
use std::iter::FromIterator;
use std::string::String;
use std::vec::Vec;

pub struct Rules {
    file_path: &'static str,
    pattern: HashSet<String>,
}

impl Rules {
    pub fn new(file_path: &'static str) -> Rules {
        let mut rules = Rules {
            file_path,
            pattern: HashSet::new(),
        };
        rules.load().expect("failed to load rules");
        rules
    }

    pub fn load(&mut self) -> Result<()> {
        let raw_content = fs::read(self.file_path)?;
        let lines = String::from_utf8(raw_content)
            .unwrap()
            .split("\n")
            .map(str::to_string)
            .collect::<Vec<String>>();

        self.pattern = HashSet::from_iter(lines);

        return Ok(());
    }

    pub fn add(&mut self, rules: &Vec<&str>) -> Result<()> {
        let mut options = fs::OpenOptions::new();
        let mut file: File = options.append(true).create(true).open(self.file_path)?;
        for r in rules {
            if self.pattern.insert(r.to_string()) {
                file.write([r, "\n"].concat().as_bytes())?;
            }
        }

        println!("rules: {:?}", self.get());
        return Ok(());
    }

    pub fn get(&self) -> Vec<&str> {
        self.pattern
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
    }
}
