use std::fs::{self, File};
use std::io::{Result, Write};
use std::string::String;
use std::vec::Vec;

pub struct Rules {
    file_path: &'static str,
}

impl Rules {
    pub fn new(file_path: &'static str) -> Rules {
        return Rules {
            file_path: file_path,
        };
    }

    pub fn add(&self, rules: &Vec<&str>) -> Result<()> {
        let mut options = fs::OpenOptions::new();
        let mut file: File = options.append(true).create(true).open(self.file_path)?;
        for r in rules {
            file.write(r.as_bytes())?;
            file.write("\n".as_bytes())?;
        }

        println!("rules: {:?}", self.get());
        return Ok(());
    }

    pub fn get(&self) -> Result<Vec<String>> {
        let v: Vec<u8> = fs::read(self.file_path)?;
        let s = String::from_utf8(v).unwrap();
        let split: Vec<String> = s.split("\n").map(str::to_string).collect();
        return Ok(split);
    }
}
