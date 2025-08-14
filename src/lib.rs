pub struct Config {
    pub file_path: String,
    pub ignore_case: bool,
    patterns: Vec<Box<dyn Matcher>>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            file_path: "".to_string(),
            ignore_case: false,
            patterns: vec![],
        }
    }

    pub fn push_matcher(&mut self, matcher: Box<dyn Matcher>) {
        self.patterns.push(matcher);
    }
}

pub trait Matcher {
    fn match_line(&self, conf: &Config, line: &str) -> bool;
}

pub struct StaticMatcher {
    query: String,
}

impl StaticMatcher {
    pub fn new(query: &str) -> Box<dyn Matcher> {
        Box::new(StaticMatcher {
            query: query.to_string()
        })
    }
}

impl Matcher for StaticMatcher {
    fn match_line(&self, conf: &Config, line: &str) -> bool {
        if conf.ignore_case {
            line.to_lowercase().contains(&self.query.to_lowercase())
        } else {
            line.contains(&self.query)
        }
    }
}

pub fn search<'a>(config: &Config, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        for pattern in config.patterns.iter() {
            if pattern.as_ref().match_line(&config, line) {
                results.push(line);
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
        let query = StaticMatcher::new("duct");
        cfg.push_matcher(query);
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(&cfg, contents));
        assert_eq!(1, cfg.patterns.len());
    }
}
