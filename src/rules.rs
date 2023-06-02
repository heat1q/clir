use anyhow::{Context, Result};
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
use std::time::Instant;
use std::vec::Vec;

use crate::path::{canonicalize, PathTree};

pub(crate) struct Rules<'a> {
    file_path: &'a Path,
    collection: HashSet<RawPattern>,
}

impl<'a> Rules<'a> {
    pub(crate) fn new(file_path: &'a Path) -> Result<Rules<'a>> {
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

                    if let Ok(pattern) = RawPattern::from_str(&line.to_string()) {
                        self.collection.insert(pattern);
                    }
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

    pub(crate) fn add(&mut self, patterns: Vec<String>) -> Result<()> {
        patterns
            .into_iter()
            .filter_map(canonicalize)
            .map(RawPattern::new)
            .for_each(|p| {
                self.collection.insert(p);
            });

        log::info!("rules: {:?}", self.get());
        self.write()?;

        Ok(())
    }

    pub(crate) fn remove(&mut self, patterns: Vec<String>) -> Result<()> {
        patterns
            .iter()
            .filter_map(|p| RawPattern::from_str(p).ok())
            .for_each(|p| {
                self.collection.remove(&p);
            });

        self.write()?;

        Ok(())
    }

    pub(crate) fn write(&self) -> Result<()> {
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

    pub(crate) fn get(&self) -> Vec<&RawPattern> {
        self.collection.iter().collect()
    }

    pub(crate) fn expand_patterns(&self, path_tree: &mut PathTree) -> Vec<Pattern> {
        // patterns can be expanded concurrently
        let patterns: Vec<Pattern> = self
            .get()
            .par_iter()
            .filter_map(|pattern| pattern.expand_glob())
            .collect();

        // insert the paths into the tree
        patterns
            .iter()
            .for_each(|pattern| pattern.insert(path_tree));

        // get the size of the individual patterns after
        // all path are inserted into the tree because
        // now we can remove all the overlapping paths
        let mut patterns: Vec<Pattern> = patterns
            .into_iter()
            .par_bridge()
            .map(|p| p.filter_and_get_size(path_tree))
            .filter(|p| !p.is_empty())
            .collect();

        patterns.par_sort_by_key(|p| p.get_size_cached());
        patterns
    }

    pub(crate) fn clean(&self, patterns: &Vec<Pattern>) -> Result<()> {
        let _n = patterns.par_iter().filter_map(|p| p.clean().ok()).count();

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct RawPattern {
    pattern: PathBuf,
}

impl PartialEq for RawPattern {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for RawPattern {}

impl RawPattern {
    fn new(pattern: PathBuf) -> Self {
        Self { pattern }
    }

    pub(crate) fn expand_glob(&self) -> Option<Pattern<'_>> {
        let glob_paths = glob::glob(self.pattern.to_str()?).ok()?;
        let start = Instant::now();

        let paths: Vec<PathBuf> = glob_paths
            .flatten()
            .filter_map(|path| fs::canonicalize(path).ok())
            .collect();

        log::trace!(
            "new pattern {:?}: num_paths: {}, time: {:?}",
            self.pattern,
            paths.len(),
            Instant::elapsed(&start)
        );

        Some(Pattern::new(self.pattern.as_path(), paths))
    }
}

impl Hash for RawPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pattern.hash(state)
    }
}

impl fmt::Display for RawPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern.to_str().ok_or(fmt::Error {})?)
    }
}

impl FromStr for RawPattern {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = RawPattern {
            pattern: PathBuf::from(s),
        };
        Ok(pattern)
    }
}

pub(crate) struct Pattern<'a> {
    pattern: &'a Path,
    paths: Vec<PathBuf>,
    size: Option<u64>,
}

impl<'a> Pattern<'a> {
    pub(crate) fn new(pattern: &'a Path, paths: Vec<PathBuf>) -> Self {
        Self {
            pattern,
            paths,
            size: None,
        }
    }

    pub(crate) fn filter_and_get_size(mut self, path_tree: &PathTree) -> Self {
        let mut size = 0;
        self.paths = self
            .paths
            .into_iter()
            .filter_map(|path| path_tree.get_size_at(&path).map(|sz| (path, sz)))
            .map(|(path, sz)| {
                size += sz;
                path
            })
            .collect();

        self.size = size.into();
        self
    }

    pub(crate) fn insert(&self, path_tree: &mut PathTree) {
        let start = Instant::now();
        self.paths.iter().for_each(|path| {
            path_tree.insert(path);
        });

        log::trace!(
            "pattern insert: {:?}, time: {:?}",
            self.pattern,
            Instant::elapsed(&start)
        );
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.size.map_or(true, |s| s == 0)
    }

    pub(crate) fn get_size_cached(&self) -> Option<u64> {
        self.size
    }

    pub(crate) fn num_files(&self) -> usize {
        self.paths.iter().filter(|p| p.is_file()).count()
    }

    pub(crate) fn num_dirs(&self) -> usize {
        self.paths.iter().filter(|p| p.is_dir()).count()
    }

    pub(crate) fn clean(&self) -> Result<()> {
        for path in &self.paths {
            if path.is_dir() {
                if let Err(err) = fs::remove_dir_all(path) {
                    log::warn!("failed to remove directory {path:?}: {err}");
                    continue;
                }
                log::info!("removed directory {path:?}");
            } else {
                if let Err(err) = fs::remove_file(path) {
                    log::warn!("failed to remove file {path:?}: {err}");
                    continue;
                }
                log::info!("removed file {path:?}");
            }
        }

        log::trace!("cleaned pattern {self}");

        Ok(())
    }
}

impl fmt::Display for Pattern<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern.to_str().ok_or(fmt::Error {})?)
    }
}

impl AsRef<Path> for Pattern<'_> {
    fn as_ref(&self) -> &Path {
        self.pattern
    }
}
