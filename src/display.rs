use std::fmt::Display;

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
        let fmt_str = match self {
            &Self::None(sz) => self.format("B", sz),
            &Self::Kilo(sz) => self.format("K", sz / 1_000),
            &Self::Mega(sz) => self.format("M", sz / 1_000_000),
            &Self::Giga(sz) => self.format("G", sz / 1_000_000_000),
            &Self::Tera(sz) => self.format("T", sz / 1_000_000_000_0000),
        };
        write!(f, "{fmt_str}")
    }
}
