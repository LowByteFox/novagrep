use novagrep::{search, Config, StaticMatcher};
use std::{env, error::Error, fs, process};


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = Config::build(&args);

    if let Err(err) = run(config) {
        eprintln!("Application error: {err}");
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;

    for line in search(&config, &contents) {
        println!("{line}");
    }

    Ok(())
}

fn usage() {
    eprintln!("usage: novagrep [OPTIONS]... patterns [file...]

Pattern Selection:
\t-E, --extended\t\tMatch using regex, for grep compat
\t-F, --fixed\t\tMatch using fixed strings, for grep compat
\t-e, --extend\t\tAdd additional patterns using regex
\t-f, --from-file\t\tLoad pattern list for match from a file
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
\t             \t\tthat accepts a single argument.");
    process::exit(1);
}

trait ArgParsing {
     fn build(args: &[String]) -> Config;
}

impl ArgParsing for Config {
     fn build(args: &[String]) -> Config {
        if args.len() < 2 {
            usage();
        }

        let query = &args[0];
        let file_path = args[1].clone();
        let ignore_case = env::var("IGNORE_CASE").is_ok();
        let query = StaticMatcher::new(query);

        let mut cfg = Config::new();
        cfg.ignore_case = ignore_case;
        cfg.file_path = file_path;

        cfg.push_matcher(query);

        cfg
    }
}
