use std::error::Error;

use super::rules::Rules;

pub struct Config {
    pub rules: Rules,
}

impl Config {
    pub fn new(rules_file: &'static str) -> Result<Config, Box<dyn Error>> {
        let cfg = Config {
            rules: Rules::new(rules_file)?,
        };
        Ok(cfg)
    }
}
