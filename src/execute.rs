//! Command execution module
//!
//! Executes translated Windows commands and captures output.

use std::process::{Command, Stdio};
use std::io::{self, Write};

use crate::args::LsArgs;
use crate::translate::Translation;

#[derive(Debug)]
pub enum Backend {
    Cmd,
    PowerShell,
}

impl Backend {
    pub fn detect() -> Self {
        // Default to cmd for speed, PowerShell for complex formatting
        Backend::Cmd
    }
}

pub struct ExecutionResult {
    pub success: bool,
    pub exit_code: i32,
}

/// Execute the translation and return output
pub fn execute(args: &LsArgs, translation: &Translation) -> io::Result<ExecutionResult> {
    // Determine backend
    let backend = if args.use_powershell {
        Backend::PowerShell
    } else if args.use_cmd {
        Backend::Cmd
    } else if args.human_readable || args.long_format {
        // PowerShell handles these better
        Backend::PowerShell
    } else {
        Backend::detect()
    };

    let (program, cmd_args, command_str) = match backend {
        Backend::Cmd => (
            "cmd.exe",
            vec!["/C", &translation.cmd_command],
            &translation.cmd_command,
        ),
        Backend::PowerShell => (
            "powershell.exe",
            vec!["-NoProfile", "-Command", &translation.powershell_command],
            &translation.powershell_command,
        ),
    };

    // If explain mode, just print and don't execute
    if args.explain {
        println!("Command (cmd.exe):    {}", translation.cmd_command);
        println!("Command (PowerShell): {}", translation.powershell_command);
        println!();
        println!("Description: {}", translation.description);
        return Ok(ExecutionResult {
            success: true,
            exit_code: 0,
        });
    }

    // If native mode, just output the command
    if args.native {
        match backend {
            Backend::Cmd => println!("{}", translation.cmd_command),
            Backend::PowerShell => println!("{}", translation.powershell_command),
        }
        return Ok(ExecutionResult {
            success: true,
            exit_code: 0,
        });
    }

    // If teach mode, print command first
    if args.teach {
        eprintln!("Executing: {}", command_str);
        eprintln!("---");
    }

    // Execute the command
    let output = Command::new(program)
        .args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Print output
    if !stdout.is_empty() {
        print!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    // Flush to ensure output appears
    io::stdout().flush().ok();
    io::stderr().flush().ok();

    Ok(ExecutionResult {
        success: output.status.success(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Print help message
pub fn print_help() {
    println!(
        r#"ls-wrapper - A lightweight ls transpiler for Windows

USAGE:
    ls [OPTIONS] [PATH]...

OPTIONS:
    -l              Long listing format
    -a, --all       Show hidden files (including . and ..)
    -A, --almost-all  Show hidden files (excluding . and ..)
    -h, --human-readable  Human-readable file sizes
    -1              One entry per line
    -R, --recursive  List subdirectories recursively
    -d, --directory  List directories themselves, not contents
    -F, --classify  Append indicator (/ for directories)
    -s              Show file size (compatibility flag)

    -t              Sort by modification time
    -S              Sort by file size
    -r, --reverse   Reverse sort order
    -U              Do not sort

    --color[=WHEN]  Colorize output (always, never, auto)

EDUCATIONAL FLAGS:
    --explain       Show Windows translation without executing
    --teach         Execute AND show what command was run
    --native        Output only the Windows command (for scripting)
    --rosetta       Show Unix → Windows command cheatsheet
    --tree          Tree view of directory structure
    --powershell    Force PowerShell backend
    --cmd           Force cmd.exe backend

ALIASES:
    ll              Same as ls -l  (rename binary to ll.exe)
    la              Same as ls -la (rename binary to la.exe)
    l               Same as ls -F  (rename binary to l.exe)

EXAMPLES:
    ls              List current directory
    ls -la          Long format, show hidden
    ls -lR ./src    Recursive, long format
    ls --explain -la  See how -la translates to Windows

    ls --native -la   Output: dir /A .

ABOUT:
    Translates Unix ls commands to Windows dir/PowerShell equivalents.
    Tiny, fast, educational.

    Source: https://github.com/yourusername/ls-wrapper
"#
    );
}

pub fn print_version() {
    println!("ls-wrapper {}", env!("CARGO_PKG_VERSION"));
}

/// Print the Rosetta Stone cheatsheet
pub fn print_rosetta() {
    println!(
        r#"
┌─────────────────────────────────────────────────────────────────────────────┐
│                        UNIX → WINDOWS CHEAT SHEET                           │
├─────────────────────┬─────────────────────┬─────────────────────────────────┤
│ Unix                │ cmd.exe             │ PowerShell                      │
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ ls                  │ dir /B              │ Get-ChildItem                   │
│ ls -l               │ dir                 │ Get-ChildItem | Format-Table    │
│ ls -la              │ dir /A              │ Get-ChildItem -Force            │
│ ls -lt              │ dir /O-D            │ gci | Sort LastWriteTime -Desc  │
│ ls -lS              │ dir /O-S            │ gci | Sort Length -Desc         │
│ ls -R               │ dir /S              │ Get-ChildItem -Recurse          │
│ ls -1               │ dir /B              │ (gci).Name                      │
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ cat file            │ type file           │ Get-Content file                │
│ head -n 10 file     │ (no equivalent)     │ Get-Content file -First 10      │
│ tail -n 10 file     │ (no equivalent)     │ Get-Content file -Last 10       │
│ grep pattern file   │ findstr pattern file│ Select-String pattern file      │
│ grep -r pattern .   │ findstr /S pattern *│ Get-ChildItem -Recurse | sls pat│
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ pwd                 │ cd                  │ Get-Location  (or pwd)          │
│ cd dir              │ cd dir              │ Set-Location dir  (or cd)       │
│ cd ~                │ cd %USERPROFILE%    │ cd ~                            │
│ mkdir dir           │ mkdir dir           │ New-Item -Type Directory dir    │
│ rm file             │ del file            │ Remove-Item file                │
│ rm -rf dir          │ rmdir /S /Q dir     │ Remove-Item dir -Recurse -Force │
│ cp src dst          │ copy src dst        │ Copy-Item src dst               │
│ mv src dst          │ move src dst        │ Move-Item src dst               │
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ touch file          │ type nul > file     │ New-Item file                   │
│ chmod +x file       │ (no equivalent)     │ (no equivalent)                 │
│ which cmd           │ where cmd           │ Get-Command cmd                 │
│ whoami              │ whoami              │ whoami  (or $env:USERNAME)      │
│ clear               │ cls                 │ Clear-Host  (or cls)            │
│ history             │ doskey /history     │ Get-History                     │
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ tree                │ tree                │ tree                            │
│ find . -name "*.rs" │ dir /S /B *.rs      │ gci -Recurse -Filter *.rs       │
│ wc -l file          │ find /c /v "" file  │ (Get-Content file).Count        │
│ diff file1 file2    │ fc file1 file2      │ Compare-Object (gc f1) (gc f2)  │
│ echo $VAR           │ echo %VAR%          │ echo $env:VAR                   │
│ export VAR=val      │ set VAR=val         │ $env:VAR = "val"                │
└─────────────────────┴─────────────────────┴─────────────────────────────────┘

Aliases:  ll = ls -l  │  la = ls -la  │  l = ls -F

Tip: Use --explain with any ls command to see its Windows translation!
     Example: ls --explain -laR
"#
    );
}
