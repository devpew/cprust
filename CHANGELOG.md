# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- `-u` / `--update` — only copy when source is newer than destination
- `--dry-run` — show what would be copied without actually copying
- `-i` / `--interactive` — prompt before overwriting existing files
- `-b` / `--backup` — create `.bak` backup before overwriting
- `--exclude=PAT` — exclude files/directories matching a glob pattern
- `-l` — create hard link instead of copying (Unix)
- `-T` / `--no-target-dir` — treat destination as a normal file
- `--version` — show version number
- Summary statistics after copy: files, dirs, skipped, bytes, time, speed
- Colored output for files, directories, and status messages
- Code coverage reporting in CI (codecov-action)
- Windows CI support
- `cargo audit` security check in CI
- Benchmark suite using criterion
- GitHub issue and PR templates
- CONTRIBUTING.md guide
- LICENSE file (MIT)
- `CHANGELOG.md`

### Changed
- Version bumped to 0.3.0
- Improved exclude pattern matching with globset
- Better error messages for canonicalization failures

## [0.2.0] - 2024-06-11

### Added
- Recursive directory copying with `-r` / `-R`
- Verbose mode `-v`
- Quiet mode `-q`
- Preserve metadata `-p`
- Follow symbolic links `-L` / `-P`
- No-clobber `-n`
- Force overwrite `-f`
- `--parents` for directory structure recreation
- `--progress` progress bar
- Multiple source files support
- Relative and absolute path support
- 17 integration tests
- GitHub Actions CI (clippy, tests, fmt)
- Chinese README (README-cn.md)

### Changed
- Refactored into modules: cli.rs, copy.rs, utils.rs

## [0.1.0] - 2024-06-06

### Added
- Initial release
- Basic single file copy
- Relative path support
- Same-file protection
