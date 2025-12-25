//! Translation engine: ls flags → Windows commands
//!
//! Translates Unix ls arguments into equivalent Windows dir or PowerShell commands.

use crate::args::LsArgs;

#[derive(Debug, Clone)]
pub struct Translation {
    pub cmd_command: String,
    pub powershell_command: String,
    pub description: String,
}

pub fn translate(args: &LsArgs) -> Translation {
    let cmd_command = build_dir_command(args);
    let powershell_command = build_powershell_command(args);
    let description = build_description(args);

    Translation {
        cmd_command,
        powershell_command,
        description,
    }
}

fn build_dir_command(args: &LsArgs) -> String {
    let mut cmd = String::from("dir");
    let mut flags = Vec::new();

    // /A - show hidden files (like -a)
    if args.all || args.almost_all {
        flags.push("/A");
    }

    // /S - recursive (like -R)
    if args.recursive {
        flags.push("/S");
    }

    // /AD - directories only (like -d, sort of)
    if args.directory {
        flags.push("/AD");
    }

    // /B - bare format (like -1)
    // Only use /B if we want simple output and not long format
    if args.one_per_line && !args.long_format {
        flags.push("/B");
    }

    // Sorting options
    // Note: ls shows newest/largest FIRST by default, dir shows oldest/smallest first
    // So we invert: ls -t = dir /O-D (descending date)
    if args.no_sort {
        // No sort flag for dir, it's default
    } else if args.sort_by_time {
        if args.reverse {
            flags.push("/OD"); // -tr = oldest first (ascending)
        } else {
            flags.push("/O-D"); // -t = newest first (descending)
        }
    } else if args.sort_by_size {
        if args.reverse {
            flags.push("/OS"); // -Sr = smallest first (ascending)
        } else {
            flags.push("/O-S"); // -S = largest first (descending)
        }
    } else if args.reverse {
        flags.push("/O-N"); // Reverse name
    }

    // Build command string
    for flag in flags {
        cmd.push(' ');
        cmd.push_str(flag);
    }

    // Add paths
    for path in &args.paths {
        cmd.push(' ');
        let win_path = to_windows_path(path);
        if win_path.contains(' ') {
            cmd.push('"');
            cmd.push_str(&win_path);
            cmd.push('"');
        } else {
            cmd.push_str(&win_path);
        }
    }

    cmd
}

fn build_powershell_command(args: &LsArgs) -> String {
    let mut cmd = String::from("Get-ChildItem");
    let mut params = Vec::new();

    // -Force - show hidden files (like -a)
    if args.all || args.almost_all {
        params.push("-Force");
    }

    // -Recurse (like -R)
    if args.recursive {
        params.push("-Recurse");
    }

    // -Directory (like -d)
    if args.directory {
        params.push("-Directory");
    }

    // Add parameters
    for param in params {
        cmd.push(' ');
        cmd.push_str(param);
    }

    // Add paths
    for path in &args.paths {
        cmd.push_str(" -Path ");
        let win_path = to_windows_path(path);
        if win_path.contains(' ') {
            cmd.push('"');
            cmd.push_str(&win_path);
            cmd.push('"');
        } else {
            cmd.push_str(&win_path);
        }
    }

    // Add sorting
    if !args.no_sort {
        if args.sort_by_time {
            cmd.push_str(" | Sort-Object LastWriteTime");
            if !args.reverse {
                cmd.push_str(" -Descending");
            }
        } else if args.sort_by_size {
            cmd.push_str(" | Sort-Object Length");
            if !args.reverse {
                cmd.push_str(" -Descending");
            }
        } else if args.reverse {
            cmd.push_str(" | Sort-Object Name -Descending");
        }
    }

    // Format output for -l equivalent
    if args.long_format {
        cmd.push_str(" | Format-Table Mode, LastWriteTime, Length, Name -AutoSize");
    } else if args.one_per_line {
        cmd.push_str(" | Select-Object -ExpandProperty Name");
    }

    cmd
}

fn build_description(args: &LsArgs) -> String {
    let mut parts = Vec::new();

    if args.all {
        parts.push("show hidden files");
    }
    if args.long_format {
        parts.push("long format");
    }
    if args.recursive {
        parts.push("recursive");
    }
    if args.sort_by_time {
        parts.push("sort by time");
    }
    if args.sort_by_size {
        parts.push("sort by size");
    }
    if args.reverse {
        parts.push("reverse order");
    }
    if args.human_readable {
        parts.push("human-readable sizes");
    }

    if parts.is_empty() {
        "list directory contents".to_string()
    } else {
        format!("list directory contents ({})", parts.join(", "))
    }
}

/// Convert Unix-style paths to Windows-style
fn to_windows_path(path: &str) -> String {
    let mut result = path.to_string();

    // Convert forward slashes to backslashes
    result = result.replace('/', "\\");

    // Handle home directory (~)
    if result.starts_with('~') {
        if let Ok(home) = std::env::var("USERPROFILE") {
            result = result.replacen('~', &home, 1);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_translation() {
        let args = LsArgs::default();
        let trans = translate(&args);
        assert!(trans.cmd_command.starts_with("dir"));
    }

    #[test]
    fn test_la_translation() {
        let mut args = LsArgs::default();
        args.long_format = true;
        args.all = true;
        let trans = translate(&args);
        assert!(trans.cmd_command.contains("/A"));
    }
}
