use regex::{Regex, RegexBuilder};

use crate::Config;

pub trait Matcher {
    fn match_line(&self, conf: &Config, line: &str) -> bool;
}

pub struct StringMatcher(String);

pub struct RegexMatcher(Regex);

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

impl RegexMatcher {
    pub fn new(pattern: &str, case_insensitive: bool) -> Box<dyn Matcher> {
        let mut binding = RegexBuilder::new(pattern);
        let builder = binding
            .case_insensitive(case_insensitive)
            .unicode(true);
        Box::new(RegexMatcher(builder.build().unwrap()))
    }
}

impl Matcher for RegexMatcher {
    fn match_line(&self, _conf: &Config, line: &str) -> bool {
        self.0.is_match(line)
    }
}
