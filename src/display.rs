use crate::rules::Pattern;
use anyhow::{Ok, Result};
use io::Write;
use rayon::prelude::*;
use std::{convert::TryInto, fmt::Display, io, path::Path};

pub fn format_patterns(workdir: &Path, patterns: Vec<&Pattern>) {
    let mut stdout = io::stdout();

    let total_size: u64 = patterns
        .par_iter()
        .map(|pattern| pattern.get_size().unwrap_or(0))
        .sum();

    if total_size == 0 {
        write_boxed(&mut stdout, "There is nothing to do :)").unwrap();
        return;
    }

    let mut patterns: Vec<&Pattern> = patterns
        .into_iter()
        .filter(|pattern| !pattern.is_empty())
        .collect();
    patterns.par_sort_by_cached_key(|k| k.get_size());
    patterns
        .iter()
        .for_each(|pattern| write_pattern(&mut stdout, workdir, pattern, total_size).unwrap());

    let num_files: u64 = patterns.par_iter().map(|p| p.num_files()).sum();
    let num_dirs: u64 = patterns.par_iter().map(|p| p.num_dirs()).sum();

    let mut summary = String::with_capacity(2 << 7);
    summary.push_str(&format!("[||||||||]  {}    ", SizeUnit::new(total_size)));
    summary.push_str(&match (num_files, num_dirs) {
        (0, _) => format!("{num_dirs} directory(ies) to be freed"),
        (_, 0) => format!("{num_files} file(s) to be freed"),
        (_, _) => format!("{num_files} file(s) and {num_dirs} directory(ies) to be freed"),
    });

    write_boxed(&mut stdout, &summary).unwrap();
    stdout.flush().unwrap();
}

fn write_pattern(
    mut w: impl Write,
    workdir: &Path,
    pattern: &Pattern,
    total_size: u64,
) -> Result<()> {
    const SCALE: i32 = 8;
    let mut quota =
        (pattern.get_size().unwrap_or(0) as f64 / total_size as f64 * SCALE as f64) as i32;
    quota = std::cmp::min(SCALE, quota + 1);
    let used = "|".repeat(quota.try_into().unwrap_or(0));
    let free = " ".repeat((SCALE - quota).try_into().unwrap_or(0));
    writeln!(
        w,
        "  [{used}{free}]  {}    {} ({} , {} )",
        SizeUnit::new(pattern.get_size().unwrap_or(0)),
        format_relative_path(workdir, pattern),
        pattern.num_files(),
        pattern.num_dirs(),
    )?;
    Ok(())
}

fn write_boxed<W: Write>(w: &'_ mut W, text: &str) -> Result<()> {
    let width = text.len() + 2;
    let horizontal = "━".repeat(width);
    writeln!(w, "┏{horizontal}┓")?;
    writeln!(w, "┃ {text} ┃")?;
    writeln!(w, "┗{horizontal}┛")?;
    Ok(())
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
