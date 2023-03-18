use std::str;
use std::{
    fs::{self, OpenOptions},
    io::BufWriter,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Default)]
pub struct MockFiles {
    config_file: PathBuf,
    dirs: Vec<PathBuf>,
    files: Vec<PathBuf>,
}

impl MockFiles {
    pub fn new() -> Self {
        Self {
            config_file: "/tmp/.clir".into(),
            dirs: vec![],
            files: vec![],
        }
    }

    pub fn add_config(mut self, path: &str, patterns: Vec<String>) -> io::Result<Self> {
        Self::write_config_file(path, patterns)?;
        self.config_file = path.into();
        Ok(self)
    }

    pub fn add_dir(mut self, path: &str) -> io::Result<Self> {
        fs::create_dir_all(path)?;
        self.dirs.push(path.into());
        Ok(self)
    }

    pub fn add_file(mut self, path: &str, n: usize) -> io::Result<Self> {
        Self::write_file(path, n)?;
        self.files.push(path.into());
        Ok(self)
    }

    pub fn write_config_file(path: &str, patterns: Vec<String>) -> io::Result<()> {
        let _ = fs::remove_file(path);
        let file = OpenOptions::new().write(true).create(true).open(path)?;

        let mut file_buf = BufWriter::new(file);
        patterns
            .iter()
            .map(|p| file_buf.write([p.as_str(), "\n"].concat().as_bytes()))
            .collect::<io::Result<Vec<usize>>>()?;

        file_buf.flush()
    }

    fn write_file(path: &str, n: usize) -> io::Result<usize> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .unwrap()
            .write(&[0].repeat(n))
    }
}

impl Drop for MockFiles {
    fn drop(&mut self) {
        let _ = self
            .dirs
            .iter()
            .map(fs::remove_dir_all)
            .collect::<io::Result<Vec<()>>>();
        let _ = self
            .files
            .iter()
            .map(fs::remove_file)
            .collect::<io::Result<Vec<()>>>();
    }
}

pub enum Item {
    Break,
    Entry(String, String),
    Summary(String),
}

impl Item {
    pub fn from_stdout(input: &str) -> Self {
        if input.starts_with(' ') {
            let input = input.split(']').nth(1).unwrap();
            let mut split = input.split(' ').filter(|s| !s.is_empty());
            let size_fmt = split.next().unwrap();
            let pattern = split.next().unwrap();
            Self::Entry(pattern.to_owned(), size_fmt.to_owned())
        } else if input.starts_with('┃') {
            let input = input.split(']').nth(1).unwrap();
            let mut split = input.split(' ').filter(|s| !s.is_empty());
            let size_fmt = split.next().unwrap();
            Self::Summary(size_fmt.to_owned())
        } else {
            Self::Break
        }
    }

    pub fn matches_pattern(&self, pattern: &str) -> bool {
        match self {
            Self::Entry(p, _) => p == pattern,
            _ => false,
        }
    }

    pub fn size_fmt(&self) -> Option<&str> {
        match self {
            Self::Entry(_, s) | Self::Summary(s) => Some(s),
            _ => None,
        }
    }

    pub fn pattern(&self) -> Option<&str> {
        match self {
            Self::Entry(s, _) => Some(s),
            _ => None,
        }
    }
}

pub struct OutputParser {
    items: Vec<Item>,
}

impl OutputParser {
    pub fn from_stdout(stdout: &[u8]) -> Self {
        let items = str::from_utf8(stdout)
            .unwrap()
            .split('\n')
            .map(Item::from_stdout)
            .collect();
        Self { items }
    }

    pub fn get_match(&self, pattern: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.matches_pattern(pattern))
    }

    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items
            .iter()
            .filter(|i| matches!(i, Item::Entry(_, _)))
            .nth(index)
    }

    pub fn summary(&self) -> Option<&Item> {
        self.items.iter().find(|i| matches!(i, Item::Summary(_)))
    }
}

#[macro_export]
macro_rules! assert_pattern {
    ($parser:ident, $pat:literal) => {
        assert!($parser.get_match($pat).and_then(|p| p.size_fmt()).is_some())
    };
    ($parser:ident, $pat:literal, $sz:literal) => {
        assert_eq!(
            $parser.get_match($pat).and_then(|p| p.size_fmt()),
            Some($sz)
        )
    };
}

#[macro_export]
macro_rules! assert_pattern_at {
    ($parser:ident, $i:literal, $pat:literal) => {
        assert_eq!($parser.get($i).and_then(|p| p.pattern()), Some($pat))
    };
    ($parser:ident, $i:literal, $pat:expr) => {
        assert_eq!($parser.get($i).and_then(|p| p.pattern()), $pat)
    };
}

#[macro_export]
macro_rules! assert_pattern_summary {
    ($parser:ident, $sz:literal) => {
        assert_eq!($parser.summary().and_then(|p| p.size_fmt()), Some($sz))
    };
}
