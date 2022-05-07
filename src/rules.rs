use core::cmp::Eq;
use core::hash::Hash;
use glob::glob;
use std::collections::HashSet;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::{ParseError, String};
use std::vec::Vec;

use crate::fs as cfs;

pub struct Rules {
    file_path: &'static str,
    collection: HashSet<Pattern>,
}

impl Rules {
    pub fn new(file_path: &'static str) -> io::Result<Rules> {
        let mut rules = Rules {
            file_path,
            collection: HashSet::new(),
        };
        rules.load()?;
        Ok(rules)
    }

    fn load(&mut self) -> io::Result<()> {
        if let Ok(file_content) = fs::read(self.file_path) {
            if let Ok(lines) = String::from_utf8(file_content) {
                let lines = lines.split("\n");

                for line in lines {
                    // ignore emtpy lines
                    if line == "" {
                        continue;
                    }

                    if let Ok(pattern) = Pattern::new(line.to_string()) {
                        self.collection.insert(pattern);
                    }
                }
            } else {
                println!("cannot parse file content");
            }

            return Ok(());
        } else {
            // create empty rules file if not exist
            fs::write(self.file_path, &[])
        }
    }

    pub fn add(&mut self, patterns: Vec<String>) -> io::Result<()> {
        for pattern in patterns {
            if let Ok(pattern) = Pattern::new(pattern) {
                self.collection.insert(pattern);
            }
        }

        println!("rules: {:?}", self.get());
        self.write()?;

        Ok(())
    }

    pub fn remove(&mut self, patterns: Vec<String>) -> io::Result<()> {
        for pattern in patterns {
            self.collection
                .remove(&Pattern::from_str(pattern.as_str()).unwrap());
        }

        self.write()?;

        Ok(())
    }

    pub fn write(&self) -> io::Result<()> {
        fs::remove_file(self.file_path)?;
        let mut options = fs::OpenOptions::new();
        let mut file: File = options.append(true).create(true).open(self.file_path)?;
        for r in self.get() {
            file.write([r.to_string().as_str(), "\n"].concat().as_bytes())?;
        }
        Ok(())
    }

    pub fn get(&self) -> Vec<&Pattern> {
        self.collection.iter().collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pattern {
    pattern: String,
    paths: Option<Vec<PathBuf>>,
}

impl Pattern {
    fn new(pattern: String) -> Result<Self, Box<dyn Error>> {
        let glob_paths = glob(&pattern)?;

        let mut paths: Vec<PathBuf> = Vec::new();
        for path in glob_paths {
            if let Ok(path) = path {
                paths.push(path);
            }
        }

        Ok(Pattern {
            pattern,
            paths: Some(paths),
        })
    }

    pub fn get_size(&self) -> Option<u64> {
        let paths = self.paths.as_ref()?;

        let mut total: u64 = 0;
        for path in paths {
            if let Ok(size) = cfs::get_size(path) {
                total += size;
            }
        }

        Some(total)
    }
}

impl Hash for Pattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pattern.hash(state)
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

impl FromStr for Pattern {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Pattern {
            pattern: String::from(s),
            paths: None,
        })
    }
}
