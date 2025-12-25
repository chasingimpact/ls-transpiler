//! ls-wrapper: A lightweight ls transpiler for Windows
//!
//! Translates Unix ls commands to native Windows equivalents.
//! Tiny, fast, educational.

mod args;
mod execute;
mod translate;

use std::env;
use std::process::{Command, ExitCode, Stdio};
use std::path::Path;
use std::io::{self, Write};

use args::LsArgs;
use execute::{execute, print_help, print_version, print_rosetta};
use translate::translate;

/// Detect alias from program name (ll, la, l)
fn get_alias_flags(program_name: &str) -> Option<&'static str> {
    let name = Path::new(program_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match name {
        "ll" => Some("-l"),
        "la" => Some("-la"),
        "l" => Some("-F"),
        _ => None,
    }
}

fn main() -> ExitCode {
    // Get command line arguments
    let mut args: Vec<String> = env::args().collect();

    // Check for alias (ll, la, l)
    if let Some(alias_flags) = args.first().and_then(|s| get_alias_flags(s)) {
        // Insert the alias flags after program name
        args.insert(1, alias_flags.to_string());
    }

    let ls_args = match LsArgs::parse(&args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("ls-wrapper: {}", e);
            eprintln!("Try 'ls --help' for more information.");
            return ExitCode::from(1);
        }
    };

    // Handle help and version
    if ls_args.help {
        print_help();
        return ExitCode::SUCCESS;
    }

    if ls_args.version {
        print_version();
        return ExitCode::SUCCESS;
    }

    // Handle --rosetta cheatsheet
    if ls_args.rosetta {
        print_rosetta();
        return ExitCode::SUCCESS;
    }

    // Handle --tree (run Windows tree command)
    if ls_args.tree {
        return run_tree(&ls_args);
    }

    // Translate ls arguments to Windows commands
    let translation = translate(&ls_args);

    // Execute the translation
    match execute(&ls_args, &translation) {
        Ok(result) => {
            if result.success {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(result.exit_code as u8)
            }
        }
        Err(e) => {
            eprintln!("ls-wrapper: execution error: {}", e);
            ExitCode::from(1)
        }
    }
}

/// Run the Windows tree command
fn run_tree(args: &LsArgs) -> ExitCode {
    let path = args.paths.first().map(|s| s.as_str()).unwrap_or(".");

    // tree /F shows files, tree /A uses ASCII
    let output = Command::new("cmd.exe")
        .args(["/C", "tree", "/F", path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            io::stdout().write_all(&output.stdout).ok();
            io::stderr().write_all(&output.stderr).ok();
            if output.status.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("ls-wrapper: failed to run tree: {}", e);
            ExitCode::from(1)
        }
    }
}
