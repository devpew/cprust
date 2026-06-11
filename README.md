# cprust

A feature-rich file and directory copy utility written in Rust. Supports recursive copying, symbolic links, metadata preservation, progress bars, multiple sources, hard links, backup, exclude patterns, and more.

## Features

- Copy files and directories recursively
- Symbolic link support (copy as symlink or follow)
- Preserve file metadata (timestamps, permissions)
- Progress bar for large files and directories
- Multiple source files to a single destination directory
- Recreate full directory structure with `--parents`
- No-clobber and force overwrite modes
- Update mode — only copy when source is newer (`-u`)
- Dry run — preview what would be copied (`--dry-run`)
- Interactive mode — prompt before overwrite (`-i`)
- Backup existing files before overwrite (`-b`)
- Exclude files by glob pattern (`--exclude`)
- Hard link instead of copy (`-l`)
- No target directory mode (`-T`)
- Verbose and quiet output modes
- Colored output for files, directories, and status
- Summary statistics (files, dirs, bytes, speed)
- Relative and absolute path support
- Protection against copying a file or directory onto itself

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
| `-u` | Update — only copy when source is newer than destination |
| `-i` | Interactive — prompt before overwriting |
| `-b` | Backup — create `.bak` file before overwriting |
| `-l` | Create hard link instead of copying (Unix) |
| `-T` | No target directory — treat destination as a file |
| `--parents` | Recreate full directory structure |
| `--progress` | Show progress bar for large files |
| `--dry-run` | Show what would be copied without copying |
| `--exclude=PAT` | Exclude files/dirs matching glob pattern |
| `--version` | Show version |
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

**Dry run — preview what would be copied:**

```bash
cprust -rv --dry-run mydir /tmp/
```

**Update — only copy newer files:**

```bash
cprust -ru mydir /backup/
```

**Backup before overwrite:**

```bash
cprust -b file.txt /tmp/
```

**Exclude log files:**

```bash
cprust -rv --exclude='*.log' mydir /tmp/
```

**Create hard link:**

```bash
cprust -l file.txt /tmp/file_link
```

**No target directory:**

```bash
cprust -rT mydir /tmp/
# Copies contents of mydir directly into /tmp/
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

**Interactive mode:**

```bash
cprust -i file.txt /tmp/
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

## Benchmarks

```bash
cargo bench
```

## CI

This project uses GitHub Actions for continuous integration:

- **Clippy** — lint checks with `-D warnings`
- **Tests** — full test suite on Ubuntu, macOS, and Windows
- **Format** — `cargo fmt` style checks
- **Coverage** — code coverage with `cargo-llvm-cov`
- **Audit** — security audit with `cargo audit`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
