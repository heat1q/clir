use crate::{path::PathTree, rules::Pattern};
use ansi_term::{ANSIString, Color, Style};
use anyhow::Result;
use core::fmt;
use io::Write;
use std::{
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

const SCALE: usize = 10;
const NUM_TABLE_COLUMS: usize = 5;
const BLOCK_CHAR: char = '\u{1fb0b}';

struct FormatTable {
    entries: Vec<TableEntry>,
    total_size: u64,
}

impl FormatTable {
    fn new(patterns: &[Pattern], workdir: &Path, absolute_path: bool, total_size: u64) -> Self {
        let num_files = patterns.iter().map(|p| p.num_files()).sum();
        let num_dirs = patterns.iter().map(|p| p.num_dirs()).sum();

        let mut entries: Vec<TableEntry> = Vec::with_capacity(patterns.len() + 2);
        entries.push(TableEntry::heading());

        patterns.iter().for_each(|p| {
            let entry = TableEntry::from_pattern(p, total_size, workdir, absolute_path);
            entries.push(entry)
        });

        let summary = TableEntry::summary(total_size, num_files, num_dirs);
        entries.push(summary);

        Self {
            entries,
            total_size,
        }
    }

    fn format(&self, w: &mut impl io::Write) -> io::Result<()> {
        if self.total_size == 0 {
            write_boxed(w, "There is nothing to do :)")?;
            return Ok(());
        }

        let mut column_widths = vec![0usize; NUM_TABLE_COLUMS];
        for entry in self.entries.iter() {
            entry.row.iter().enumerate().for_each(|(i, c)| {
                let chars_count = c.as_ref().map(chars_count).unwrap_or(0);
                column_widths[i] = column_widths[i].max(chars_count);
            })
        }

        let [entries @ .., summary] = &self.entries[..] else {
            unreachable!()
        };

        for entry in entries {
            write!(w, "  ")?;
            entry.format(w, &column_widths)?;
            writeln!(w)?;
        }

        let mut buf = Vec::new();
        summary.format(&mut buf, &column_widths)?;
        write_boxed(w, std::str::from_utf8(&buf).unwrap_or(""))?;

        Ok(())
    }
}

struct TableEntry {
    pub row: [Option<ANSIString<'static>>; NUM_TABLE_COLUMS],
}

impl TableEntry {
    fn from_pattern(
        pattern: &Pattern,
        total_size: u64,
        workdir: &Path,
        absolute_path: bool,
    ) -> Self {
        let row: [Option<ANSIString<'_>>; 5] = [
            Some(Self::format_quota(pattern, total_size)),
            Some(
                SizeUnit::new(pattern.get_size_cached().unwrap_or(0), true)
                    .to_string()
                    .into(),
            ),
            Self::format_dirs(pattern.num_dirs()).map(|s| s.into()),
            Self::format_files(pattern.num_files()).map(|s| s.into()),
            Some(
                format_pattern(pattern, workdir, absolute_path)
                    .to_string_lossy()
                    .to_string()
                    .into(),
            ),
        ];

        Self { row }
    }

    fn heading() -> Self {
        Self {
            row: [
                Some(Style::new().bold().paint("Share")),
                Some(Style::new().bold().paint("Size")),
                Some(Style::new().bold().paint("Dirs")),
                Some(Style::new().bold().paint("Files")),
                Some(Style::new().bold().paint("Path")),
            ],
        }
    }

    fn summary(total_size: u64, num_files: usize, num_dirs: usize) -> Self {
        Self {
            row: [
                Some(format!("[{}]", BLOCK_CHAR.to_string().repeat(SCALE)).into()),
                Some(SizeUnit::new(total_size, true).to_string().into()),
                Self::format_dirs(num_dirs).map(|s| s.into()),
                Self::format_files(num_files).map(|s| s.into()),
                Some("files and directories can be removed".into()),
            ],
        }
    }

    fn format(&self, w: &mut impl io::Write, column_widths: &[usize]) -> io::Result<()> {
        const PADDING: usize = 2;
        for (i, col) in self.row.iter().enumerate() {
            match col {
                Some(col) if i < NUM_TABLE_COLUMS - 1 => {
                    let padding = " ".repeat(column_widths[i] - chars_count(col) + PADDING);
                    write!(w, "{col}{padding}")
                }
                Some(col) if i == NUM_TABLE_COLUMS - 1 => {
                    write!(w, "{col}")
                }
                None if column_widths[i] > 0 && i < NUM_TABLE_COLUMS - 1 => {
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

    fn format_quota(pattern: &Pattern, total_size: u64) -> ANSIString<'static> {
        let quota = (pattern.get_size_cached().unwrap_or(0) as f64 / total_size as f64
            * SCALE as f64) as usize;
        let quota = std::cmp::min(SCALE, quota + 1);
        let diff = SCALE - quota;
        let used = BLOCK_CHAR.to_string().repeat(quota);
        let free = " ".repeat(diff);

        const SCALE_50: usize = SCALE * 3 / 5;
        const SCALE_80: usize = SCALE * 9 / 10;
        let color = match quota {
            _ if quota > SCALE_80 => Color::Red,
            SCALE_50..=SCALE_80 => Color::Yellow,
            _ => Color::Green,
        };

        format!("[{}]", color.paint(format!("{used}{free}"))).into()
    }
}

fn chars_count(s: &ANSIString<'_>) -> usize {
    let mut count = 0;
    let mut is_ansi_style = false;
    for c in s.chars() {
        if c == '\x1b' {
            is_ansi_style = true;
            continue;
        }
        if is_ansi_style && c == 'm' {
            is_ansi_style = false;
            continue;
        }
        if is_ansi_style {
            continue;
        }
        count += 1;
    }

    count as usize
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
