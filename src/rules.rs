use anyhow::{anyhow, Context, Result};
use core::cmp::Eq;
use core::hash::Hash;
use rayon::prelude::*;
use std::collections::HashSet;
use std::convert::From;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::{ParseError, String};
use std::sync::{Mutex, RwLock};
use std::time::Instant;
use std::vec::Vec;

use crate::path::{canonicalize, PathTree};

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
        if let Ok(file_content) = fs::read(self.file_path) {
            if let Ok(lines) = String::from_utf8(file_content) {
                for line in lines.split('\n') {
                    // ignore emtpy lines
                    if line.is_empty() {
                        continue;
                    }

                    let pattern = Pattern::from(line.to_string());
                    self.collection.insert(pattern);
                }
            } else {
                anyhow::bail!("failed to parse rules file content")
            }

            Ok(())
        } else {
            // create empty rules file if not exist
            fs::write(self.file_path, []).context("failed to create rules file")
        }
    }

    pub fn add(&mut self, patterns: Vec<String>) -> Result<()> {
        patterns
            .into_iter()
            .filter_map(canonicalize)
            .map(Pattern::new)
            .for_each(|p| {
                self.collection.insert(p);
            });

        log::info!("rules: {:?}", self.get());
        self.write()?;

        Ok(())
    }

    pub fn remove(&mut self, patterns: Vec<String>) -> Result<()> {
        patterns
            .iter()
            .filter_map(|p| Pattern::from_str(p).ok())
            .for_each(|p| {
                self.collection.remove(&p);
            });

        self.write()?;

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        fs::remove_file(self.file_path)?;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.file_path)?;

        let mut file_buf = BufWriter::new(file);
        for r in self.get() {
            let _n = file_buf.write([r.to_string().as_str(), "\n"].concat().as_bytes())?;
        }

        file_buf.flush()?;

        Ok(())
    }

    pub fn get(&self) -> Vec<&Pattern> {
        self.collection.iter().collect()
    }

    pub fn clean(&self, verbose_mode: bool) -> Result<()> {
        let _n = self
            .collection
            .par_iter()
            .filter_map(|p| p.clean(verbose_mode).ok())
            .count();

        Ok(())
    }
}

#[derive(Debug)]
pub struct Pattern {
    pattern: PathBuf,
    paths: RwLock<Option<Vec<PathBuf>>>,
    size: Mutex<Option<u64>>,
    num_files: Mutex<u64>,
    num_dirs: Mutex<u64>,
}

impl Default for Pattern {
    fn default() -> Self {
        Pattern {
            pattern: PathBuf::new(),
            paths: RwLock::new(None),
            size: Mutex::new(None),
            num_files: Mutex::new(0),
            num_dirs: Mutex::new(0),
        }
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for Pattern {}

impl Pattern {
    fn new(pattern: PathBuf) -> Self {
        Self {
            pattern,
            ..Self::default()
        }
    }

    fn count_files<'a>(
        path: &'a PathBuf,
        num_files: &'a mut u64,
        num_dirs: &'a mut u64,
    ) -> &'a PathBuf {
        if path.is_file() {
            *num_files += 1;
        } else if path.is_dir() {
            *num_dirs += 1;
        }
        path
    }

    pub fn expand_glob(&self, path_tree: &PathTree) -> Result<()> {
        // no need to expand if the paths are already covered
        if path_tree.contains_subpath(&self.pattern) {
            return Ok(());
        }

        let glob_paths = glob::glob(
            self.pattern
                .to_str()
                .ok_or_else(|| anyhow!("invalid pattern"))?,
        )?;
        let start = Instant::now();

        let mut num_files: u64 = 0;
        let mut num_dirs: u64 = 0;
        let paths: Vec<PathBuf> = glob_paths
            .flatten()
            .filter_map(|path| {
                Self::count_files(&path, &mut num_files, &mut num_dirs);
                fs::canonicalize(&path).ok()
            })
            .collect();

        log::debug!(
            "new pattern {:?}: num_paths: {}, time: {:?}",
            self.pattern,
            paths.len(),
            Instant::elapsed(&start)
        );

        let _ = self.paths.write().unwrap().insert(paths);

        Ok(())
    }

    pub fn insert(&self, path_tree: &mut PathTree) -> Result<()> {
        let start = Instant::now();
        self.paths
            .read()
            .unwrap()
            .as_ref()
            .ok_or_else(|| anyhow!("no paths given"))?
            .iter()
            .for_each(|path| {
                path_tree.insert(path);
            });

        log::debug!(
            "pattern insert: {:?}, time: {:?}",
            self.pattern,
            Instant::elapsed(&start)
        );

        Ok(())
    }

    pub fn get_size(&self, path_tree: &PathTree) -> Option<u64> {
        let mut num_files: u64 = 0;
        let mut num_dirs: u64 = 0;

        let start = Instant::now();

        let size: u64 = self
            .paths
            .read()
            .unwrap()
            .as_ref()?
            .iter()
            .filter_map(|path| {
                let size = path_tree.get_size_at(path);
                if size.unwrap_or(0) > 0 {
                    Self::count_files(path, &mut num_files, &mut num_dirs);
                }
                size
            })
            .sum();

        log::debug!(
            "pattern get_size: {:?}: {}, time: {:?}",
            self.pattern,
            size,
            Instant::elapsed(&start)
        );

        let _ = self.size.lock().unwrap().insert(size);
        *self.num_files.lock().unwrap() = num_files;
        *self.num_dirs.lock().unwrap() = num_dirs;

        Some(size)
    }

    pub fn get_size_cached(&self) -> Option<u64> {
        *self.size.lock().unwrap()
    }

    pub fn num_files(&self) -> u64 {
        *self.num_files.lock().unwrap()
    }

    pub fn num_dirs(&self) -> u64 {
        *self.num_dirs.lock().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.num_files() + self.num_dirs() == 0
    }

    pub fn clean(&self, verbose_mode: bool) -> Result<()> {
        for path in self.paths.read().unwrap().as_ref().unwrap() {
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

            if verbose_mode {
                println!("deleted {}", path.to_str().unwrap_or(""));
            }
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
        write!(f, "{}", self.pattern.to_str().ok_or(fmt::Error {})?)
    }
}

impl FromStr for Pattern {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = Pattern {
            pattern: PathBuf::from(s),
            ..Pattern::default()
        };
        Ok(pattern)
    }
}

impl AsRef<Path> for Pattern {
    fn as_ref(&self) -> &Path {
        Path::new(&self.pattern)
    }
}

impl From<String> for Pattern {
    fn from(value: String) -> Self {
        Self::new(PathBuf::from(value))
    }
}
