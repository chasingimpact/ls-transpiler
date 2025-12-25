# ls

A lightweight `ls` command for Windows. Translates Unix ls syntax to native Windows commands.

**340 KB** binary. Zero dependencies.

## Install

```
cargo install --git https://github.com/chasingimpact/ls-transpiler
```

Or download from [Releases](https://github.com/chasingimpact/ls-transpiler/releases) and add to PATH.

## Usage

```bash
ls              # list files
ls -la          # long format + hidden
ls -lt          # sort by time
ls -lS          # sort by size
ls -R           # recursive
```

## Educational Features

```bash
ls --explain -la    # show Windows translation without running
ls --teach -la      # run and show what command was executed
ls --rosetta        # full Unix-to-Windows cheatsheet
```

## Aliases

Copy `ls.exe` to create shortcuts:
- `ll.exe` = `ls -l`
- `la.exe` = `ls -la`

## Build

```
cargo build --release
```

## License

MIT
