pub struct Config {
    sources: Vec<String>,
    patterns: Vec<Box<dyn Matcher>>,
    pub all_regex: bool,
    pub all_strings: bool,
    pub ignore_case: bool,
    pub quiet: bool,
    pub list_matched_files: bool,
    pub show_count: bool,
    pub show_line_numbers: bool,
    pub suppress_missing: bool,
    pub invert_match: bool,
}

impl Config {
    pub fn new() -> Config {
        Config {
            sources: vec![],
            patterns: vec![],
            all_regex: false,
            all_strings: false,
            ignore_case: false,
            quiet: false,
            list_matched_files: false,
            show_count: false,
            show_line_numbers: false,
            suppress_missing: false,
            invert_match: false,
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

#[derive(Debug)]
pub struct SearchResult<'a> {
    pub line: &'a str,
    pub linenr: usize,
}

impl<'a> PartialEq for SearchResult<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line
    }
}

pub fn search<'a>(config: &Config, contents: &'a str) -> Vec<SearchResult<'a>> {
    let mut linenr: usize = 1;
    let mut results = Vec::new();

    for line in contents.lines() {
        for pattern in config.patterns.iter() {
            if pattern.as_ref().match_line(&config, line) {
                results.push(SearchResult { line, linenr });
                break;
            }
        }
        linenr += 1;
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
        let contents = "
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(
            vec![SearchResult {
                line: "safe, fast, productive.",
                linenr: 1
            }],
            search(&cfg, contents)
        );
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

        assert_eq!(
            vec![
                SearchResult {
                    line: "Rust:",
                    linenr: 0,
                },
                SearchResult {
                    line: "Trust me.",
                    linenr: 0,
                }
            ],
            search(&cfg, contents)
        );
    }
}
