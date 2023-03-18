use crate::{path::PathTree, rules::Pattern};
use anyhow::Result;
use core::fmt;
use io::Write;
use std::{
    convert::TryInto,
    fmt::Display,
    io, mem,
    path::{Path, PathBuf},
};

pub(crate) fn format_patterns(
    workdir: &Path,
    path_tree: &PathTree,
    patterns: &[Pattern],
    absolute_path: bool,
) -> Result<()> {
    let mut stdout = io::stdout();
    let total_size = path_tree.get_size().unwrap_or(0);

    let table = FormatTable::new(patterns, workdir, absolute_path, total_size);

    write!(stdout, "{table}")?;
    stdout.flush()?;

    Ok(())
}

const SCALE: i32 = 8;

struct FormatTable<'a> {
    entries: Vec<TableEntry<'a>>,
    workdir: &'a Path,
    absolute_path: bool,
    total_size: u64,
}

impl<'a> FormatTable<'a> {
    fn new(
        patterns: &'a [Pattern],
        workdir: &'a Path,
        absolute_path: bool,
        total_size: u64,
    ) -> Self {
        let entries = patterns
            .iter()
            .map(|p| TableEntry::from_pattern(p, total_size))
            .collect();

        Self {
            entries,
            workdir,
            absolute_path,
            total_size,
        }
    }

    fn format_pattern(&self, pattern: &Pattern) -> PathBuf {
        let path = pattern.as_ref();
        if self.absolute_path {
            return path.to_owned();
        }

        let index = self
            .workdir
            .components()
            .zip(path.components())
            .filter(|(a, b)| a == b)
            .count();

        // number of dirs to go up in relation to workdir
        let num_dirs_up = self.workdir.components().count() - index;
        if num_dirs_up > 2 {
            // if the distance is to big, just return the absolute path
            return path.into();
        }

        //let rel_path = (0..num_dirs_up).map(|_| "..").chain
        let path = path
            .iter()
            .enumerate()
            .filter(|&(i, _)| i >= index)
            .filter_map(|(_, p)| p.to_str());

        (0..num_dirs_up).map(|_| "..").chain(path).collect()
    }
}

impl Display for FormatTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.total_size == 0 {
            write_boxed(f, "There is nothing to do :)")?;
            return Ok(());
        }

        self.entries.iter().try_for_each(|entry| {
            let used = "|".repeat(entry.quota.try_into().unwrap_or(0));
            let free = " ".repeat((SCALE - entry.quota).try_into().unwrap_or(0));
            writeln!(
                f,
                "  [{used}{free}]  {}    {} ({} , {} )",
                SizeUnit::new(entry.pattern.get_size_cached().unwrap_or(0), true),
                self.format_pattern(entry.pattern).to_string_lossy(),
                entry.pattern.num_files(),
                entry.pattern.num_dirs(),
            )
        })?;

        let num_files = self.entries.iter().map(|e| e.pattern.num_files()).sum();
        let num_dirs = self.entries.iter().map(|e| e.pattern.num_dirs()).sum();

        let mut summary = String::with_capacity(2 << 7);
        summary.push_str(&format!(
            "[||||||||]  {}    ",
            SizeUnit::new(self.total_size, true)
        ));
        summary.push_str(&match (num_files, num_dirs) {
            (0, _) => format!("{num_dirs} directory(ies) to be freed"),
            (_, 0) => format!("{num_files} file(s) to be freed"),
            (_, _) => format!("{num_files} file(s) and {num_dirs} directory(ies) to be freed"),
        });

        write_boxed(f, &summary)
    }
}

struct TableEntry<'a> {
    pub pattern: &'a Pattern<'a>,
    pub quota: i32,
}

impl<'a> TableEntry<'a> {
    fn from_pattern(pattern: &'a Pattern, total_size: u64) -> Self {
        let quota = (pattern.get_size_cached().unwrap_or(0) as f64 / total_size as f64
            * SCALE as f64) as i32;
        let quota = std::cmp::min(SCALE, quota + 1);

        Self { pattern, quota }
    }
}

fn write_boxed<W: fmt::Write>(w: &mut W, text: &str) -> fmt::Result {
    let width = text.len() + 2;
    let horizontal = "━".repeat(width);
    writeln!(w, "┏{horizontal}┓")?;
    writeln!(w, "┃ {text} ┃")?;
    writeln!(w, "┗{horizontal}┛")
}

#[repr(u8)]
enum SizeUnitRaw {
    None = 0,
    Kilo,
    Mega,
    Giga,
    Tera,
}

impl Display for SizeUnitRaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Kilo => write!(f, "K"),
            Self::Mega => write!(f, "M"),
            Self::Giga => write!(f, "G"),
            Self::Tera => write!(f, "T"),
        }
    }
}

struct SizeUnit {
    val: u64,
    unit: SizeUnitRaw,
    base2: bool,
}

impl SizeUnit {
    pub(crate) fn new(val: u64, base2: bool) -> Self {
        let base = if base2 { 1024 } else { 1000 };
        let mut i: u8 = 0;
        let mut v = val;
        loop {
            if v / base == 0 {
                break;
            }
            v /= base;
            i += 1;
        }

        Self {
            base2,
            val: v,
            unit: match i {
                1 => SizeUnitRaw::Kilo,
                2 => SizeUnitRaw::Mega,
                3 => SizeUnitRaw::Giga,
                5 => SizeUnitRaw::Tera,
                _ => SizeUnitRaw::None,
            },
        }
    }
}

impl Display for SizeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.val;
        let unit = &self.unit;
        if let SizeUnitRaw::None = self.unit {
            return write!(f, "{val}B");
        }
        let i = if self.base2 { "i" } else { "" };

        if val < 10 {
            write!(f, "{:.2}{unit}{i}B", val as f64)
        } else if val < 100 {
            write!(f, "{:.1}{unit}{i}B", val as f64)
        } else {
            return write!(f, "{val}{unit}{i}B");
        }
    }
}
