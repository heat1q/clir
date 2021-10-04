use super::rules::Rules;

pub struct Config {
    pub rules: Rules,
}

impl Config {
    pub fn new(rules_file: &'static str) -> Config {
        return Config {
            rules: Rules::new(rules_file),
        };
    }
}
