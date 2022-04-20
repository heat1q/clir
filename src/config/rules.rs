use std::collections::HashSet;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::iter::FromIterator;
use std::string::String;
use std::vec::Vec;

pub struct Rules {
    file_path: &'static str,
    pattern: HashSet<String>,
}

impl Rules {
    pub fn new(file_path: &'static str) -> io::Result<Rules> {
        let mut rules = Rules {
            file_path,
            pattern: HashSet::new(),
        };
        rules.load()?;
        Ok(rules)
    }

    pub fn load(&mut self) -> io::Result<()> {
        let raw_content = fs::read(self.file_path)?;
        let lines = String::from_utf8(raw_content)
            .unwrap()
            .split("\n")
            .map(str::to_string)
            .collect::<Vec<String>>();

        self.pattern = HashSet::from_iter(lines);
        Ok(())
    }

    pub fn add(&mut self, rules: Vec<String>) -> io::Result<()> {
        for r in rules {
            self.pattern.insert(r);
        }
        println!("rules: {:?}", self.get());
        self.write()?;
        Ok(())
    }

    pub fn remove(&mut self, rules: Vec<String>) -> io::Result<()> {
        for r in rules {
            self.pattern.remove(&r);
        }
        self.write()?;
        Ok(())
    }

    pub fn write(&self) -> io::Result<()> {
        fs::remove_file(self.file_path)?;
        let mut options = fs::OpenOptions::new();
        let mut file: File = options.append(true).create(true).open(self.file_path)?;
        for r in self.get() {
            file.write([r, "\n"].concat().as_bytes())?;
        }
        Ok(())
    }

    pub fn get(&self) -> Vec<&str> {
        self.pattern
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
    }
}
