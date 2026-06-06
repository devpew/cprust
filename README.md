# cprust

A minimal file and directory copy utility written in Rust. Supports recursive directory copying, relative and absolute paths, automatic destination detection, and same-file/directory protection.

## Features

- Copy files and directories recursively
- `-r` / `-R` flag support for recursive directory copying
- Automatic directory detection — if the destination is an existing directory, the source name is appended
- Support for both relative and absolute paths
- Protection against copying a file or directory onto itself
- Protection against overwriting a file with a directory
- Clear error messages for missing files, identical source/destination, and invalid operations

## Installation

### From source

```bash
git clone https://github.com/devpew/cprust.git
cd cprust
cargo build --release
```

The binary will be available at `target/release/cprust`.

### Install globally

```bash
cargo install --path .
```

## Usage

```bash
cprust [-r] <source> <destination>
```

### Options

| Flag | Description |
|------|-------------|
| `-r`, `-R` | Copy directories recursively (required for directories) |

### Examples

**Copy a file to a new location:**

```bash
cprust file.txt /tmp/file.txt
```

**Copy a file into a directory:**

```bash
cprust file.txt /tmp/
```

**Copy a directory recursively:**

```bash
cprust -r mydir /tmp/
```

**Copy a directory to a new location:**

```bash
cprust -R ./source_dir ./backup_dir
```

**Copy using relative paths:**

```bash
cprust file.txt ./backup/
```

**Copy using absolute paths:**

```bash
cprust /home/user/docs/file.txt /var/backups/file.txt
```

## Error Handling

The tool returns a non-zero exit code on errors:

| Error | Description |
|-------|-------------|
| `No such file or directory` | Source file or directory does not exist |
| `are the same file` | Source and destination resolve to the same file |
| `are the same directory` | Source and destination resolve to the same directory |
| `cannot overwrite file with directory` | Attempting to copy a directory over an existing file |

## Requirements

- Rust 1.75+ (Edition 2024)

## License

MIT
