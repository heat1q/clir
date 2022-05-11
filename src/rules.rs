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
    num_files: u64,
    num_dirs: u64,
}

impl Default for Pattern {
    fn default() -> Self {
        Pattern {
            pattern: "".to_owned(),
            paths: None,
            num_files: 0,
            num_dirs: 0,
        }
    }
}

impl Pattern {
    fn new(pattern: String) -> Result<Self, Box<dyn Error>> {
        let glob_paths = glob(&pattern)?;

        let mut num_files: u64 = 0;
        let mut num_dirs: u64 = 0;
        let mut paths: Vec<PathBuf> = Vec::new();
        for path in glob_paths {
            if let Ok(path) = path {
                if path.is_file() {
                    num_files += 1;
                } else if path.is_dir() {
                    num_dirs += 1;
                }

                paths.push(path);
            }
        }

        Ok(Pattern {
            pattern,
            paths: Some(paths),
            num_files,
            num_dirs,
        })
    }

    pub fn get_size(&self) -> Option<u64> {
        let paths = self.paths.as_ref()?;

        let mut visited: HashSet<PathBuf> = HashSet::with_capacity(paths.len());

        let mut size: u64 = 0;
        for path in paths {
            size += match self.get_path_size(path.to_path_buf(), &mut visited) {
                Some(sz) => sz,
                None => 0,
            }
        }

        Some(size)
    }

    fn get_path_size(&self, path: PathBuf, visited: &mut HashSet<PathBuf>) -> Option<u64> {
        // don't get the size for already visited paths
        if visited.contains(&path) {
            return None;
        }

        if !path.is_dir() {
            let size = path.metadata().ok()?.len();
            visited.insert(path);

            return Some(size);
        }

        let mut size: u64 = 0;

        for entry in fs::read_dir(&path).ok()? {
            let path = entry.ok()?.path();

            size += match self.get_path_size(path, visited) {
                Some(sz) => sz,
                None => 0,
            }
        }

        Some(size)
    }

    pub fn num_files(&self) -> u64 {
        self.num_files
    }

    pub fn num_dirs(&self) -> u64 {
        self.num_dirs
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
        let mut pattern = Pattern::default();
        pattern.pattern = String::from(s);
        Ok(pattern)
    }
}
