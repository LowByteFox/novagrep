use getargs::{Opt, Options};
use novagrep::{matchers::RegexMatcher, search, Config, StringMatcher};
use std::{env, error::Error, fs, io, io::ErrorKind, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Argument parsing error: {err}");
        process::exit(1);
    });

    if let Err(err) = run(config) {
        eprintln!("Application error: {err}");
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let sources = config.get_sources();
    if sources.len() == 1 {
        let source = &sources[0];
        handle_source(&config, source, false)?;
    } else {
        for source in sources {
            handle_source(&config, source, true)?;
        }
    }

    Ok(())
}

fn handle_stdin(config: &Config, has_next: bool) -> Result<(), Box<dyn Error>> {
    let mut linenr = 1;
    loop {
        let mut buf = String::new();

        io::stdin().read_line(&mut buf)?;

        let trimmed = buf.trim();
        match_string(config, "(stdin)", trimmed, has_next, linenr)?;
        linenr += 1;
    }
}

fn handle_source(config: &Config, source: &str, has_next: bool) -> Result<(), Box<dyn Error>> {
    if source == "-" {
        return handle_stdin(config, has_next);
    }

    let contents = match fs::read_to_string(&source) {
        Ok(v) => v,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                if config.suppress_missing {
                    return Ok(());
                }
                eprintln!("{source}: {e}");
                return Ok(());
            }
            return Err(Box::new(e));
        }
    };

    match_string(config, source, &contents, has_next, 0)
}

fn match_string(config: &Config, source: &str, content: &str, has_next: bool, linenr: usize) -> Result<(), Box<dyn Error>> {
    let matched_lines = search(&config, &content);
    let base_cond = !config.quiet && matched_lines.len() > 0;
    if base_cond && config.list_matched_files {
        println!("{source}");
        return Ok(());
    } else if base_cond && config.show_count {
        if has_next {
            print!("{source}:");
        }
        println!("{}", matched_lines.len());
        return Ok(());
    }

    for line in matched_lines {
        if !config.quiet {
            if has_next {
                print!("{source}:");
            }
            if config.show_line_numbers {
                if linenr == 0 {
                   print!("{}:", line.linenr);
                } else {
                   print!("{}:", linenr);
                }
            }
            println!("{}", line.line);
        }
    }

    Ok(())
}

fn usage() {
    eprintln!(
        "usage: novagrep [OPTIONS]... pattern [file...]

Pattern Selection:
\t-E, --extended\t\tMatch using regex, for grep compat
\t-F, --fixed\t\tMatch using fixed strings, for grep compat
\t-e, --extend pattern\tAdd additional patterns using regex
\t-f, --from-file file\tLoad pattern list for match from a file
\t-i, --ignore-case\tPerform case-insensitive matching

Output Control:
\t-c, --count\t\tWrite only the count of matching lines to stdout
\t-l, --list-files\tWrite only the names of files with matches
\t-n, --numbers\t\tPrecede each output line by its line number in
\t             \t\tthe file
\t-q, --quiet\t\tSuppress all output

Miscellaneous:
\t-h, --help\t\tPrint help message to stderr
\t-s, --suppress\t\tSuppress error messages for missing files
\t-v, --invert-match\tSelect lines that do not match
\t    --exec-match\tRun a custom command to perform matching
\t    --exec\t\tRun a command on each matching line
\t    --lisp-match\tPerform matching using a Lisp script
\t    --lisp\t\tRun a Lisp script on each matching line

Usage Details:
\tPattern Syntax\t\tFollow https://docs.rs/regex/1.11.1/regex/#syntax
\t              \t\tfor regex pattern matching.

\t              \t\tSeparate each pattern with a newline.

\tExec Command Syntax\tUse \"{{}}\" in the command; it will be replaced
\t                   \twith the matching line. End the command with \\;

\t                   \tExample:
\t                   \t  novagrep foo file.txt --exec echo {{}} \\;

\tLisp Matching\t\tYou can find list of supported expressions
\t             \t\tunder \"Included functionality\" here
\t             \t\thttps://github.com/brundonsmith/rust_lisp#included-functionality

\t             \t\tEach Lisp script must return a lambda 
\t             \t\tthat accepts a single argument."
    );
    process::exit(1);
}

trait ArgParsing {
    fn build<'a>(args: &'a [String]) -> Result<Config, Box<dyn Error + 'a>>;
}

impl ArgParsing for Config {
    fn build<'a>(args: &'a [String]) -> Result<Config, Box<dyn Error + 'a>> {
        if args.is_empty() {
            usage();
        }

        let mut has_patterns = false;
        let mut cfg = Config::new();
        let mut opts = Options::new(args.iter().map(String::as_str));

        while let Some(opt) = opts.next_opt().expect("argument parsing error") {
            match opt {
                Opt::Short('E') | Opt::Long("extended") => {
                    if cfg.all_strings {
                        return Err("Cannot use both -F and -E".into());
                    }

                    cfg.all_regex = true;
                }
                Opt::Short('F') | Opt::Long("fixed") => {
                    if cfg.all_regex {
                        return Err("Cannot use both -F and -E".into());
                    }

                    cfg.all_strings = true;
                }
                Opt::Short('e') | Opt::Long("--extend") => {
                    let patterns = opts.value()?;

                    for pattern in patterns.lines() {
                        cfg.push_matcher(
                            RegexMatcher::new(pattern, cfg.ignore_case)
                        );
                    }

                    has_patterns = true;
                }
                Opt::Short('f') | Opt::Long("--from-file") => {
                    let file = opts.value()?;

                    let patterns = fs::read_to_string(file)?;
                    for pattern in patterns.lines() {
                        if cfg.all_regex {
                            cfg.push_matcher(
                                RegexMatcher::new(pattern, cfg.ignore_case)
                            );
                        } else {
                            cfg.push_matcher(StringMatcher::new(pattern));
                        }
                    }

                    has_patterns = true;
                }
                Opt::Short('i') | Opt::Long("--ignore-case") => cfg.ignore_case = true,
                Opt::Short('c') | Opt::Long("--count") => cfg.show_count = true,
                Opt::Short('l') | Opt::Long("--list-files") => cfg.list_matched_files = true,
                Opt::Short('n') | Opt::Long("--numbers") => cfg.show_line_numbers = true,
                Opt::Short('q') | Opt::Long("--quiet") => cfg.quiet = true,
                Opt::Short('s') | Opt::Long("--suppress") => cfg.suppress_missing = true,
                Opt::Short('v') | Opt::Long("--invert-match") => cfg.invert_match = true,
                Opt::Short('h') | Opt::Long("help") => usage(),
                _ => {
                    return Err(format!("unknown flag: {}", opt.to_string()).into());
                }
            }
        }

        if !has_patterns {
            if let Some(query) = opts.next_positional() {
                if cfg.all_regex {
                    cfg.push_matcher(
                        RegexMatcher::new(query, cfg.ignore_case)
                    );
                } else {
                    cfg.push_matcher(StringMatcher::new(query));
                }
            } else {
                return Err("missing pattern for matching".to_string().into());
            }
        }

        for arg in opts.positionals() {
            cfg.push_source(arg.to_string());
        }

        if cfg.get_sources().len() == 0 {
            cfg.push_source("-".to_string());
        }

        Ok(cfg)
    }
}
