# cprust

A feature-rich file and directory copy utility written in Rust. Supports recursive copying, symbolic links, metadata preservation, progress bars, multiple sources, and more.

## Features

- Copy files and directories recursively
- Symbolic link support (copy as symlink or follow)
- Preserve file metadata (timestamps, permissions)
- Progress bar for large files and directories
- Multiple source files to a single destination directory
- Recreate full directory structure with `--parents`
- No-clobber and force overwrite modes
- Verbose and quiet output modes
- Relative and absolute path support
- Protection against copying a file or directory onto itself
- Protection against overwriting a file with a directory
- Clear error messages for all failure modes

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
cprust [OPTION]... SOURCE... DESTINATION
```

### Options

| Flag | Description |
|------|-------------|
| `-r`, `-R` | Copy directories recursively |
| `-v` | Verbose mode — print each copied file |
| `-q` | Quiet mode — suppress output |
| `-p` | Preserve file metadata (timestamps, permissions) |
| `-L` | Follow symbolic links (copy target, not link) |
| `-P` | Do not follow symbolic links (copy as symlink, default) |
| `-n` | No-clobber — do not overwrite existing files |
| `-f` | Force — overwrite existing files |
| `--parents` | Recreate full directory structure |
| `--progress` | Show progress bar for large files |
| `-h`, `--help` | Show help message |

### Examples

**Copy a file:**

```bash
cprust file.txt /tmp/file.txt
```

**Copy a file into a directory:**

```bash
cprust file.txt /tmp/
```

**Copy multiple files to a directory:**

```bash
cprust file1.txt file2.txt dir/
```

**Copy a directory recursively:**

```bash
cprust -r mydir /tmp/
```

**Copy with verbose output:**

```bash
cprust -rv mydir /tmp/
```

**Copy preserving metadata:**

```bash
cprust -rp mydir /tmp/
```

**Copy with progress bar:**

```bash
cprust --progress largefile.iso /backup/
```

**Copy without overwriting existing files:**

```bash
cprust -n file.txt /tmp/
```

**Force overwrite:**

```bash
cprust -f file.txt /tmp/existing.txt
```

**Recreate directory structure:**

```bash
cprust --parents a/b/c/file.txt /backup/
# Creates /backup/a/b/c/file.txt
```

**Follow symbolic links:**

```bash
cprust -L link.txt /tmp/
```

**Copy symbolic links as links:**

```bash
cprust -P link.txt /tmp/
```

## Error Handling

The tool returns a non-zero exit code on errors:

| Error | Description |
|-------|-------------|
| `No such file or directory` | Source file or directory does not exist |
| `are the same file` | Source and destination resolve to the same file |
| `are the same directory` | Source and destination resolve to the same directory |
| `cannot overwrite file with directory` | Attempting to copy a directory over an existing file |
| `cannot copy multiple sources to ... which does not exist` | Multiple sources require an existing directory as destination |
| `omitting directory (use -r for recursive)` | Attempting to copy a directory without `-r` flag |

## Requirements

- Rust 1.75+ (Edition 2024)

## Testing

```bash
cargo test
```

## CI

This project uses GitHub Actions for continuous integration:

- **Clippy** — lint checks with `-D warnings`
- **Tests** — full integration test suite
- **Format** — `cargo fmt` style checks

## License

MIT
