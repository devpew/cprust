# cprust

A minimal file copy utility written in Rust. Supports relative and absolute paths, directory destination detection, and same-file protection.

## Features

- Copy files between locations
- Automatic directory detection — if the destination is an existing directory, the source filename is appended
- Support for both relative and absolute paths
- Protection against copying a file onto itself
- Clear error messages for missing files, unsupported directories, and identical source/destination

## Installation

### From source

```bash
git clone https://github.com/<username>/cprust.git
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
cprust <source> <destination>
```

### Examples

Copy a file to a new location:

```bash
cprust file.txt /tmp/file.txt
```

Copy a file into a directory:

```bash
cprust file.txt /tmp/
```

Copy using relative paths:

```bash
cprust file.txt ./backup/
```

Copy using absolute paths:

```bash
cprust /home/user/docs/file.txt /var/backups/file.txt
```

## Error Handling

The tool returns non-zero exit codes for errors:

| Error | Description |
|-------|-------------|
| `No such file or directory` | Source file does not exist |
| `is a directory (not supported)` | Source is a directory (only files are supported) |
| `are the same file` | Source and destination resolve to the same file |

## Requirements

- Rust 1.75+ (Edition 2024)

## License

MIT
