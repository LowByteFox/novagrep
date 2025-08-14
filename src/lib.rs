pub struct Config {
    sources: Vec<String>,
    pub ignore_case: bool,
    pub all_regex: bool,
    pub all_strings: bool,
    patterns: Vec<Box<dyn Matcher>>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            sources: vec![],
            ignore_case: false,
            patterns: vec![],
            all_regex: false,
            all_strings: false,
        }
    }

    pub fn push_source(&mut self, source: String) {
        self.sources.push(source);
    }

    pub fn get_sources(&self) -> &Vec<String> {
        return &self.sources;
    }

    pub fn push_matcher(&mut self, matcher: Box<dyn Matcher>) {
        self.patterns.push(matcher);
    }
}

pub trait Matcher {
    fn match_line(&self, conf: &Config, line: &str) -> bool;
}

pub struct StringMatcher(String);

impl StringMatcher {
    pub fn new(query: &str) -> Box<dyn Matcher> {
        Box::new(StringMatcher(query.to_string()))
    }
}

impl Matcher for StringMatcher {
    fn match_line(&self, conf: &Config, line: &str) -> bool {
        if conf.ignore_case {
            line.to_lowercase().contains(&self.0.to_lowercase())
        } else {
            line.contains(&self.0)
        }
    }
}

pub fn search<'a>(config: &Config, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        for pattern in config.patterns.iter() {
            if pattern.as_ref().match_line(&config, line) {
                results.push(line);
                break;
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let mut cfg = Config::new();
        let query = StringMatcher::new("duct");
        cfg.push_matcher(query);
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(&cfg, contents));
        assert_eq!(1, cfg.patterns.len());
    }

    #[test]
    fn multiple_result() {
        let mut cfg = Config::new();
        let query = StringMatcher::new("rUsT");
        cfg.ignore_case = true;
        cfg.push_matcher(query);
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(vec!["Rust:", "Trust me."], search(&cfg, contents));
    }
}
