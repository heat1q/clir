use std::{fmt::Display, path::Path};

use crate::rules::Pattern;

pub fn format_patterns(workdir: &Path, patterns: Vec<&Pattern>) {
    // sort patterns by size
    let mut patterns_sorted: Vec<&Pattern> = patterns.into_iter().collect();
    patterns_sorted.sort_by_cached_key(|k| k.get_size());

    for pattern in patterns_sorted {
        if let Some(size) = pattern.get_size() {
            println!(
                "{}\t{} ({} files, {} dirs)",
                SizeUnit::new(size),
                format_relative_path(workdir, pattern),
                pattern.num_files(),
                pattern.num_dirs()
            );
        }
    }
}

fn format_relative_path(workdir: &Path, pattern: &Pattern) -> String {
    let path = pattern.to_string();
    let path: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let workdir = workdir
        .to_str()
        .unwrap()
        .split('/')
        .filter(|p| !p.is_empty());

    let index = workdir.clone().zip(&path).filter(|&(a, b)| &a == b).count();

    // number of dirs to go up in relation to workdir
    let num_dirs_up = workdir.count() - index;
    if num_dirs_up > 2 {
        // if the distance is to big, just return the absolute path
        return pattern.to_string();
    }

    let mut rel_path: Vec<&str> = (0..num_dirs_up).map(|_| "..").collect();
    path.iter()
        .enumerate()
        .filter(|&(i, _)| i >= index)
        .for_each(|(_, p)| rel_path.push(p));

    rel_path.join("/")
}

pub enum SizeUnit {
    None(u64),
    Kilo(u64),
    Mega(u64),
    Giga(u64),
    Tera(u64),
}

impl SizeUnit {
    pub fn new(size: u64) -> Self {
        let mut i = 0;
        let mut sz = size;
        while sz > 0 {
            sz /= 1_000;
            i += 1;
        }

        match i - 1 {
            1 => Self::Kilo(size),
            2 => Self::Mega(size),
            3 => Self::Giga(size),
            4 => Self::Tera(size),
            _ => Self::None(size),
        }
    }

    fn format(&self, unit: &str, size: u64) -> String {
        if size < 10 && unit != "B" {
            format!("{:.2}{unit}", size as f64)
        } else if size < 100 && unit != "B" {
            format!("{:.1}{unit}", size as f64)
        } else {
            format!("{size}{unit}")
        }
    }
}

impl Display for SizeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = match *self {
            Self::None(sz) => self.format("B", sz),
            Self::Kilo(sz) => self.format("K", sz / 1_000),
            Self::Mega(sz) => self.format("M", sz / 1_000_000),
            Self::Giga(sz) => self.format("G", sz / 1_000_000_000),
            Self::Tera(sz) => self.format("T", sz / 1_000_000_000_000),
        };
        write!(f, "{fmt_str}")
    }
}
