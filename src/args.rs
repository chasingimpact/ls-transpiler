//! Argument parsing for ls-wrapper
//! Zero-dependency argument parser for ls flags

#[derive(Debug, Default)]
pub struct LsArgs {
    // Display flags
    pub long_format: bool,      // -l
    pub all: bool,              // -a (include hidden)
    pub almost_all: bool,       // -A (hidden, but no . ..)
    pub human_readable: bool,   // -h
    pub one_per_line: bool,     // -1
    pub recursive: bool,        // -R
    pub directory: bool,        // -d (list dirs themselves, not contents)
    pub classify: bool,         // -F (append indicator)
    pub show_size: bool,        // -s (show size in blocks - on Windows, just show size)

    // Sorting flags
    pub sort_by_time: bool,     // -t
    pub sort_by_size: bool,     // -S
    pub reverse: bool,          // -r
    pub no_sort: bool,          // -U

    // Output control
    pub color: ColorOption,     // --color

    // Educational/meta flags
    pub explain: bool,          // --explain (show translation, don't run)
    pub teach: bool,            // --teach (run AND show translation)
    pub native: bool,           // --native (output Windows command only)
    pub use_powershell: bool,   // --powershell
    pub use_cmd: bool,          // --cmd

    // Help
    pub help: bool,             // --help, -?
    pub version: bool,          // --version
    pub rosetta: bool,          // --rosetta (cheatsheet)
    pub tree: bool,             // --tree (tree view)

    // Paths to list
    pub paths: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ColorOption {
    #[default]
    Auto,
    Always,
    Never,
}

impl LsArgs {
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut result = LsArgs::default();
        let mut args_iter = args.into_iter().peekable();

        // Skip program name if present
        args_iter.next();

        while let Some(arg) = args_iter.next() {
            let arg = arg.as_ref();

            if arg == "--" {
                // Everything after -- is a path
                for remaining in args_iter {
                    result.paths.push(remaining.as_ref().to_string());
                }
                break;
            } else if arg.starts_with("--") {
                // Long option
                Self::parse_long_option(&mut result, arg)?;
            } else if arg.starts_with('-') && arg.len() > 1 {
                // Short option(s) - can be combined like -la
                Self::parse_short_options(&mut result, arg)?;
            } else {
                // It's a path
                result.paths.push(arg.to_string());
            }
        }

        // Default to current directory if no paths specified
        if result.paths.is_empty() {
            result.paths.push(".".to_string());
        }

        Ok(result)
    }

    fn parse_long_option(args: &mut LsArgs, opt: &str) -> Result<(), String> {
        let opt = &opt[2..]; // Remove --

        // Handle --option=value format
        let (name, _value) = if let Some(pos) = opt.find('=') {
            (&opt[..pos], Some(&opt[pos + 1..]))
        } else {
            (opt, None)
        };

        match name {
            "all" => args.all = true,
            "almost-all" => args.almost_all = true,
            "human-readable" => args.human_readable = true,
            "recursive" => args.recursive = true,
            "directory" => args.directory = true,
            "classify" => args.classify = true,
            "reverse" => args.reverse = true,

            "color" => {
                args.color = match _value {
                    Some("always") | Some("yes") | Some("force") => ColorOption::Always,
                    Some("never") | Some("no") | Some("none") => ColorOption::Never,
                    Some("auto") | Some("tty") | Some("if-tty") | None => ColorOption::Auto,
                    Some(v) => return Err(format!("Unknown color option: {}", v)),
                };
            }

            // Educational flags
            "explain" => args.explain = true,
            "teach" => args.teach = true,
            "native" => args.native = true,
            "powershell" | "ps" => args.use_powershell = true,
            "cmd" => args.use_cmd = true,

            "help" => args.help = true,
            "version" => args.version = true,
            "rosetta" | "cheatsheet" => args.rosetta = true,
            "tree" => args.tree = true,

            _ => return Err(format!("Unknown option: --{}", name)),
        }

        Ok(())
    }

    fn parse_short_options(args: &mut LsArgs, opt: &str) -> Result<(), String> {
        // Skip the leading -
        for c in opt[1..].chars() {
            match c {
                'l' => args.long_format = true,
                'a' => args.all = true,
                'A' => args.almost_all = true,
                'h' => args.human_readable = true,
                '1' => args.one_per_line = true,
                'R' => args.recursive = true,
                'd' => args.directory = true,
                'F' => args.classify = true,
                's' => args.show_size = true,
                't' => args.sort_by_time = true,
                'S' => args.sort_by_size = true,
                'r' => args.reverse = true,
                'U' => args.no_sort = true,
                '?' => args.help = true,
                _ => return Err(format!("Unknown option: -{}", c)),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_flags() {
        let args = LsArgs::parse(["ls", "-la"]).unwrap();
        assert!(args.long_format);
        assert!(args.all);
    }

    #[test]
    fn test_path() {
        let args = LsArgs::parse(["ls", "-l", "./src"]).unwrap();
        assert!(args.long_format);
        assert_eq!(args.paths, vec!["./src"]);
    }
}
