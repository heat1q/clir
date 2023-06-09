use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::path::Path;
use std::str;
use std::{
    fs::{self, OpenOptions},
    io::BufWriter,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Default)]
pub struct MockFiles {
    test_dir: PathBuf,
    config_path: PathBuf,
}

impl MockFiles {
    pub fn new() -> Self {
        let test_dir = thread_rng()
            .sample_iter(Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>();
        let test_dir = Path::new("/tmp/clir").join(test_dir);
        fs::create_dir_all(&test_dir).unwrap();
        let config_path = test_dir.join(".clir");
        Self {
            test_dir,
            config_path,
        }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn add_config(self, name: &str, patterns: Vec<&str>) -> io::Result<Self> {
        let path = self.test_dir.join(name);
        self.write_config_file(&path, patterns)?;
        Ok(self)
    }

    pub fn add_dir(self, path: &str) -> io::Result<Self> {
        let path = self.test_dir.join(path);
        fs::create_dir_all(path)?;
        Ok(self)
    }

    pub fn add_file(self, path: &str, n: usize) -> io::Result<Self> {
        let path = self.test_dir.join(path);
        Self::write_file(&path, n)?;
        Ok(self)
    }

    pub fn write_config_file(&self, path: &Path, patterns: Vec<&str>) -> io::Result<()> {
        let _ = fs::remove_file(path);
        let file = OpenOptions::new().write(true).create(true).open(path)?;

        let mut file_buf = BufWriter::new(file);
        patterns
            .iter()
            .map(|p| self.test_dir.join(p).to_string_lossy().to_string())
            .map(|p| file_buf.write([p.as_str(), "\n"].concat().as_bytes()))
            .collect::<io::Result<Vec<usize>>>()?;

        file_buf.flush()
    }

    fn write_file(path: &Path, n: usize) -> io::Result<usize> {
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
        //let _ = fs::remove_dir_all(&self.test_dir);
    }
}

#[derive(Debug)]
pub enum Item {
    Break,
    Heading,
    Entry {
        pattern: String,
        num_dirs: usize,
        num_files: usize,
        size_fmt: String,
    },
    Summary {
        num_dirs: usize,
        num_files: usize,
        size_fmt: String,
    },
}

impl Item {
    pub fn from_stdout(input: &str) -> Self {
        for c in input.chars() {
            match c {
                ' ' => continue,
                'a'..='z' | 'A'..='Z' => return Self::Heading,
                '[' => {
                    let (_, input) = input.split_once(']').unwrap();
                    let split: Vec<&str> = input.split(' ').filter(|s| !s.is_empty()).collect();
                    return match split[..] {
                        [size_fmt, _, num_dirs, _, num_files, pattern] => Self::Entry {
                            pattern: pattern.to_string(),
                            num_dirs: num_dirs.parse().unwrap(),
                            num_files: num_files.parse().unwrap(),
                            size_fmt: size_fmt.to_string(),
                        },
                        [size_fmt, icon, count, pattern] if icon == "\u{f07b}" => Self::Entry {
                            pattern: pattern.to_string(),
                            num_dirs: count.parse().unwrap(),
                            num_files: 0,
                            size_fmt: size_fmt.to_string(),
                        },
                        [size_fmt, icon, count, pattern] if icon == "\u{f0f6}" => Self::Entry {
                            pattern: pattern.to_string(),
                            num_dirs: 0,
                            num_files: count.parse().unwrap(),
                            size_fmt: size_fmt.to_string(),
                        },
                        _ => Self::Break,
                    };
                }
                '┃' => {
                    let split: Vec<&str> = input.split(' ').filter(|s| !s.is_empty()).collect();
                    let [_, _, size_fmt, l_icon, l_count, r_icon, r_count, ..] = split[..] else {
                        return Self::Break;
                    };
                    return match (l_icon, r_icon) {
                        ("\u{f07b}", "\u{f0f6}") => Self::Summary {
                            num_dirs: l_count.parse().unwrap(),
                            num_files: r_count.parse().unwrap(),
                            size_fmt: size_fmt.to_string(),
                        },
                        ("\u{f07b}", _) => Self::Summary {
                            num_dirs: l_count.parse().unwrap(),
                            num_files: 0,
                            size_fmt: size_fmt.to_string(),
                        },
                        ("\u{f0f6}", _) => Self::Summary {
                            num_dirs: 0,
                            num_files: l_count.parse().unwrap(),
                            size_fmt: size_fmt.to_string(),
                        },
                        (_, _) => Self::Break,
                    };
                }
                _ => break,
            }
        }
        Self::Break
    }

    pub fn size_fmt(&self) -> Option<&str> {
        match self {
            Self::Entry { size_fmt, .. } | Self::Summary { size_fmt, .. } => Some(size_fmt),
            _ => None,
        }
    }

    pub fn pattern(&self) -> Option<&str> {
        match self {
            Self::Entry { pattern, .. } => Some(pattern),
            _ => None,
        }
    }

    pub fn num_dirs(&self) -> Option<usize> {
        match self {
            Self::Entry { num_dirs, .. } | Self::Summary { num_dirs, .. } => Some(*num_dirs),
            _ => None,
        }
    }

    pub fn num_files(&self) -> Option<usize> {
        match self {
            Self::Entry { num_files, .. } | Self::Summary { num_files, .. } => Some(*num_files),
            _ => None,
        }
    }
}

#[derive(Debug)]
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

    pub fn entries(&self) -> Vec<&Item> {
        self.items
            .iter()
            .filter(|i| matches!(i, Item::Entry { .. }))
            .collect()
    }

    pub fn summary(&self) -> Option<&Item> {
        self.items
            .iter()
            .find(|i| matches!(i, Item::Summary { .. }))
    }
}

#[macro_export]
macro_rules! assert_pattern_entries {
    (
        $parser:ident,
        [
            $(($pat:literal, $sz:literal, num_dirs = $num_dirs:literal, num_files = $num_files:literal)),+
            $(,)?
        ],
    ) => {
        let entries = $parser.entries();
        $(
            let e = entries.iter().find(|e| e.pattern().unwrap().ends_with($pat));
            assert!(e.is_some(), "entry should exist");
            let e = e.unwrap();
            assert_eq!(e.size_fmt(), Some($sz));
            assert_eq!(e.num_dirs(), Some($num_dirs as usize));
            assert_eq!(e.num_files(), Some($num_files as usize));
        )+
    }
}

#[macro_export]
macro_rules! assert_pattern_summary {
    ($parser:ident, $sz:literal) => {
        assert_eq!($parser.summary().and_then(|p| p.size_fmt()), Some($sz))
    };
    ($parser:ident, $sz:literal, num_dirs = $num_dirs:literal, num_files = $num_files:literal) => {
        let summary = $parser.summary();
        assert!(summary.is_some(), "summary not found");
        let summary = summary.unwrap();
        assert_eq!(summary.size_fmt(), Some($sz));
        assert_eq!(summary.num_dirs(), Some($num_dirs as usize));
        assert_eq!(summary.num_files(), Some($num_files as usize));
    };
}
