use anyhow::{Context, Result};
use core::cmp::Eq;
use core::hash::Hash;
use glob::glob;
use std::cell::RefCell;
use std::collections::HashSet;
use std::convert::From;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::{ParseError, String};
use std::vec::Vec;

pub struct Rules<'a> {
    file_path: &'a Path,
    collection: HashSet<Pattern>,
}

impl<'a> Rules<'a> {
    pub fn new(file_path: &'a Path) -> Result<Rules<'a>> {
        let mut rules = Rules {
            file_path,
            collection: HashSet::new(),
        };
        rules.load()?;

        Ok(rules)
    }

    fn load(&mut self) -> Result<()> {
        if let Ok(file_content) = fs::read(&self.file_path) {
            if let Ok(lines) = String::from_utf8(file_content) {
                for line in lines.split('\n') {
                    // ignore emtpy lines
                    if line.is_empty() {
                        continue;
                    }

                    if let Ok(pattern) = Pattern::new(line.to_string()) {
                        self.collection.insert(pattern);
                    }
                }
            } else {
                anyhow::bail!("failed to parse rules file content")
            }

            Ok(())
        } else {
            // create empty rules file if not exist
            fs::write(&self.file_path, &[]).context("failed to create rules file")
        }
    }

    pub fn add(&mut self, patterns: Vec<String>) -> Result<()> {
        for pattern in patterns {
            if let Ok(pattern) = Pattern::new(pattern) {
                self.collection.insert(pattern);
            }
        }

        println!("rules: {:?}", self.get());
        self.write()?;

        Ok(())
    }

    pub fn remove(&mut self, patterns: Vec<String>) -> Result<()> {
        for pattern in patterns {
            self.collection
                .remove(&Pattern::from_str(pattern.as_str()).unwrap());
        }

        self.write()?;

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        fs::remove_file(&self.file_path)?;
        let mut options = fs::OpenOptions::new();
        let mut file: File = options.append(true).create(true).open(&self.file_path)?;
        for r in self.get() {
            let _n = file.write([r.to_string().as_str(), "\n"].concat().as_bytes())?;
        }

        Ok(())
    }

    pub fn get(&self) -> Vec<&Pattern> {
        self.collection.iter().collect()
    }

    pub fn clean(&self) -> Result<()> {
        self.collection.iter().for_each(|pattern| {
            // TODO - error handling
            let _res = pattern.clean();
        });

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pattern {
    pattern: String,
    paths: Option<Vec<PathBuf>>,
    size: RefCell<Option<u64>>,
    num_files: u64,
    num_dirs: u64,
}

impl Default for Pattern {
    fn default() -> Self {
        Pattern {
            pattern: "".to_owned(),
            paths: None,
            size: RefCell::new(None),
            num_files: 0,
            num_dirs: 0,
        }
    }
}

impl Pattern {
    fn new(pattern: String) -> Result<Self> {
        let glob_paths = glob(&pattern)?;

        let mut num_files: u64 = 0;
        let mut num_dirs: u64 = 0;
        let paths: Vec<PathBuf> = glob_paths
            .flatten()
            .map(|path| {
                if path.is_file() {
                    num_files += 1;
                } else if path.is_dir() {
                    num_dirs += 1;
                }
                path
            })
            .collect();

        Ok(Pattern {
            pattern,
            paths: Some(paths),
            size: RefCell::new(None),
            num_files,
            num_dirs,
        })
    }

    pub fn get_size(&self) -> Option<u64> {
        if self.size.borrow().is_some() {
            return *self.size.borrow();
        }

        let paths = self.paths.as_ref()?;
        let mut visited: HashSet<PathBuf> = HashSet::with_capacity(paths.len());

        let mut size: u64 = 0;
        for path in paths {
            size += self
                .get_path_size(path.to_path_buf(), &mut visited)
                .unwrap_or(0);
        }

        self.size.replace(Some(size));
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

            size += self.get_path_size(path, visited).unwrap_or(0);
        }

        Some(size)
    }

    pub fn num_files(&self) -> u64 {
        self.num_files
    }

    pub fn num_dirs(&self) -> u64 {
        self.num_dirs
    }

    pub fn is_empty(&self) -> bool {
        self.num_files + self.num_dirs == 0
    }

    pub fn clean(&self) -> Result<()> {
        for path in self.paths.as_ref().unwrap() {
            if path.is_dir() {
                if let Err(err) = fs::remove_dir_all(path) {
                    log::warn!("failed to removed {:?}: {err}", path);
                }
                log::info!("removed dir {:?}", path);
                continue;
            }
            if let Err(err) = fs::remove_file(path) {
                log::warn!("failed to removed file {:?}: {err}", path);
            }
            log::info!("removed file {:?}", path);
        }

        log::info!("cleaned pattern {self}");

        Ok(())
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
        let pattern = Pattern {
            pattern: String::from(s),
            ..Pattern::default()
        };
        Ok(pattern)
    }
}

impl Deref for Pattern {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.pattern
    }
}
