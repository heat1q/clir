use crate::{path::PathTree, rules::Pattern};
use anyhow::Result;
use core::fmt;
use io::Write;
use std::{
    convert::TryInto,
    fmt::Display,
    io,
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

    table.format(&mut stdout)?;
    stdout.flush()?;

    Ok(())
}

const SCALE: i32 = 8;

struct FormatTable<'a> {
    entries: Vec<TableEntry>,
    workdir: &'a Path,
    absolute_path: bool,
    total_size: u64,
    num_files: usize,
    num_dirs: usize,
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
            .map(|p| TableEntry::from_pattern(p, total_size, workdir, absolute_path))
            .collect();
        let num_files = patterns.iter().map(|p| p.num_files()).sum();
        let num_dirs = patterns.iter().map(|p| p.num_dirs()).sum();

        Self {
            entries,
            workdir,
            absolute_path,
            total_size,
            num_files,
            num_dirs,
        }
    }

    fn format(&self, w: &mut impl io::Write) -> io::Result<()> {
        if self.total_size == 0 {
            write_boxed(w, "There is nothing to do :)")?;
            return Ok(());
        }
        let summary = TableEntry::summary(self.total_size, self.num_files, self.num_dirs);
        let summary = [summary];

        let mut column_widths = vec![0usize; 5];
        for entry in self.entries.iter().chain(summary.iter()) {
            entry.row.iter().enumerate().for_each(|(i, c)| {
                let chars_count = c.as_ref().map(|s| s.chars().count()).unwrap_or(0);
                column_widths[i] = column_widths[i].max(chars_count);
            })
        }

        for entry in &self.entries {
            write!(w, "  ")?;
            entry.format(w, &column_widths)?;
            writeln!(w)?;
        }

        let mut buf = Vec::new();
        summary[0].format(&mut buf, &column_widths)?;
        write_boxed(w, std::str::from_utf8(&buf).unwrap())?;

        Ok(())
    }
}

struct TableEntry {
    pub row: [Option<String>; 5],
}

impl TableEntry {
    fn from_pattern(
        pattern: &Pattern,
        total_size: u64,
        workdir: &Path,
        absolute_path: bool,
    ) -> Self {
        let quota = (pattern.get_size_cached().unwrap_or(0) as f64 / total_size as f64
            * SCALE as f64) as i32;
        let quota = std::cmp::min(SCALE, quota + 1);
        let used = "|".repeat(quota.try_into().unwrap_or(0));
        let free = " ".repeat((SCALE - quota).try_into().unwrap_or(0));

        let row: [Option<String>; 5] = [
            Some(format!("[{used}{free}]")),
            Some(SizeUnit::new(pattern.get_size_cached().unwrap_or(0), true).to_string()), //TODO: size unit
            Self::format_dirs(pattern.num_dirs()),
            Self::format_files(pattern.num_files()),
            Some(
                format_pattern(pattern, workdir, absolute_path)
                    .to_string_lossy()
                    .to_string(),
            ),
        ];

        Self { row }
    }

    fn summary(total_size: u64, num_files: usize, num_dirs: usize) -> Self {
        Self {
            row: [
                Some("[||||||||]".to_owned()),
                Some(SizeUnit::new(total_size, true).to_string()),
                Self::format_dirs(num_dirs),
                Self::format_files(num_files),
                Some("Files and directories to be removed".to_owned()),
            ],
        }
    }

    fn format(&self, w: &mut impl io::Write, column_widths: &[usize]) -> io::Result<()> {
        const PADDING: usize = 2;
        for (i, col) in self.row.iter().enumerate() {
            match col {
                Some(col) => {
                    let padding = " ".repeat(column_widths[i] - col.chars().count() + PADDING);
                    write!(w, "{col}{padding}")
                }
                None if column_widths[i] > 0 => {
                    let padding = " ".repeat(column_widths[i] + PADDING);
                    write!(w, "{padding}")
                }
                _ => Ok(()),
            }?;
        }
        Ok(())
    }

    fn format_files(num_files: usize) -> Option<String> {
        match num_files {
            0 => None,
            _ => Some(format!("\u{f0f6} {num_files}")),
        }
    }

    fn format_dirs(num_dirs: usize) -> Option<String> {
        match num_dirs {
            0 => None,
            _ => Some(format!("\u{f07b} {num_dirs}")),
        }
    }
}

fn format_pattern(pattern: &Pattern, workdir: &Path, absolute_path: bool) -> PathBuf {
    let path = pattern.as_ref();
    if absolute_path {
        return path.to_owned();
    }

    let index = workdir
        .components()
        .zip(path.components())
        .filter(|(a, b)| a == b)
        .count();

    // number of dirs to go up in relation to workdir
    let num_dirs_up = workdir.components().count() - index;
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

fn write_boxed(w: &mut impl io::Write, text: &str) -> io::Result<()> {
    let width = text.chars().count() + 2;
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
