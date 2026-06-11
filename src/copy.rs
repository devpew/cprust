use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use owo_colors::OwoColorize;

use crate::cli::Options;
use crate::utils;

pub struct CopyState {
    pub bytes_copied: u64,
    pub files_copied: u64,
    pub total_bytes: u64,
    pub dirs_copied: u64,
    pub skipped: u64,
    pub started: Instant,
}

impl CopyState {
    pub fn new() -> Self {
        Self {
            bytes_copied: 0,
            files_copied: 0,
            total_bytes: 0,
            dirs_copied: 0,
            skipped: 0,
            started: Instant::now(),
        }
    }

    pub fn print_summary(&self, quiet: bool) {
        if quiet {
            return;
        }
        let elapsed = self.started.elapsed();
        let elapsed_ms = elapsed.as_millis();
        let speed = if elapsed_ms > 0 {
            self.bytes_copied as f64 / elapsed_ms as f64 * 1000.0
        } else {
            0.0
        };
        let speed_str = utils::format_bytes(speed as u64);
        let files_str = self.files_copied.to_string();
        let dirs_str = self.dirs_copied.to_string();
        let skipped_str = self.skipped.to_string();
        let bytes_str = utils::format_bytes(self.bytes_copied);
        let speed_full = format!("{}/s", speed_str);

        eprintln!(
            "{}",
            format!(
                "Summary: {} file(s), {} dir(s), {} skipped, {} copied in {:.1}s ({})",
                files_str.cyan(),
                dirs_str.blue(),
                skipped_str.yellow(),
                bytes_str.green(),
                elapsed.as_secs_f64(),
                speed_full.green()
            )
            .bold()
        );
    }
}

pub fn copy_sources(sources: &[PathBuf], destination: &Path, opts: &Options) -> io::Result<()> {
    let dst = utils::resolve_path(destination);
    let mut state = CopyState::new();

    let matcher = if !opts.exclude.is_empty() {
        match utils::build_exclude_matcher(&opts.exclude) {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("cprust: invalid exclude pattern: {}", e);
                return Err(io::Error::other(format!("invalid exclude pattern: {}", e)));
            }
        }
    } else {
        None
    };

    if !opts.no_target_dir && !dst.exists() {
        if sources.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "cannot copy multiple sources to '{}' which does not exist",
                    dst.display()
                ),
            ));
        }
        copy_one(&sources[0], &dst, opts, &mut state, &matcher)?;
        state.print_summary(opts.quiet);
        return Ok(());
    }

    if !opts.no_target_dir && !dst.is_dir() {
        if sources.len() > 1 {
            return Err(io::Error::other(format!(
                "cannot copy multiple sources to non-directory '{}'",
                dst.display()
            )));
        }
        handle_overwrite_checks(&sources[0], &dst, opts, &mut state)?;
        copy_one(&sources[0], &dst, opts, &mut state, &matcher)?;
        state.print_summary(opts.quiet);
        return Ok(());
    }

    if opts.no_target_dir && sources.len() == 1 {
        copy_one(&sources[0], &dst, opts, &mut state, &matcher)?;
        state.print_summary(opts.quiet);
        return Ok(());
    }

    for src in sources {
        if let Some(matcher) = matcher.as_ref()
            && utils::is_excluded(src, matcher)
        {
            state.skipped += 1;
            if opts.verbose && !opts.quiet {
                println!("{}: {}", "skipped".yellow(), src.display());
            }
            continue;
        }

        let src_resolved = utils::resolve_path(src);
        let src_name = src_resolved
            .file_name()
            .ok_or_else(|| io::Error::other("cannot extract filename"))?
            .to_string_lossy()
            .to_string();

        let final_dest = if opts.parents {
            utils::build_parents_path(src, &dst)
        } else {
            dst.join(&src_name)
        };

        copy_one(src, &final_dest, opts, &mut state, &matcher)?;
    }

    state.print_summary(opts.quiet);
    Ok(())
}

fn handle_overwrite_checks(
    src: &Path,
    dst: &Path,
    opts: &Options,
    state: &mut CopyState,
) -> io::Result<()> {
    if opts.no_clobber && !opts.force {
        if !opts.quiet {
            eprintln!("{}: not overwriting '{}'", src.display(), dst.display());
        }
        state.skipped += 1;
        return Ok(());
    }
    Ok(())
}

fn copy_one(
    src: &Path,
    dst: &Path,
    opts: &Options,
    state: &mut CopyState,
    matcher: &Option<globset::GlobSet>,
) -> io::Result<()> {
    let src_resolved = utils::resolve_path(src);

    if !src_resolved.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("cannot stat '{}': No such file or directory", src.display()),
        ));
    }

    if let Some(matcher) = matcher
        && utils::is_excluded(&src_resolved, matcher)
    {
        state.skipped += 1;
        return Ok(());
    }

    let meta = if opts.follow_symlinks {
        fs::metadata(&src_resolved)
    } else {
        fs::symlink_metadata(&src_resolved)
    };

    let meta = match meta {
        Ok(m) => m,
        Err(e) => {
            return Err(io::Error::new(
                e.kind(),
                format!("cannot stat '{}': {}", src.display(), e),
            ));
        }
    };

    if meta.file_type().is_symlink() && !opts.follow_symlinks {
        return copy_symlink(&src_resolved, dst, opts, state);
    }

    if meta.is_dir() {
        if !opts.recursive {
            return Err(io::Error::other(format!(
                "omitting directory '{}' (use -r for recursive)",
                src.display()
            )));
        }
        return copy_dir_recursive(&src_resolved, dst, src, opts, state, matcher);
    }

    copy_single_file(&src_resolved, dst, opts, state)
}

fn canonicalize_or_original(p: &Path) -> PathBuf {
    if p.exists() {
        return p.canonicalize().ok().unwrap_or_else(|| p.to_path_buf());
    }
    let mut candidate = p.to_path_buf();
    let mut suffix_parts: Vec<String> = Vec::new();

    while !candidate.exists() {
        if let Some(fn_name) = candidate.file_name() {
            suffix_parts.push(fn_name.to_string_lossy().to_string());
        }
        if let Some(parent) = candidate.parent() {
            candidate = parent.to_path_buf();
        } else {
            break;
        }
    }

    if candidate.exists() {
        let base = candidate.canonicalize().ok().unwrap_or(candidate);
        suffix_parts.reverse();
        let mut result = base;
        for part in &suffix_parts {
            result = result.join(part);
        }
        result
    } else {
        p.to_path_buf()
    }
}

fn should_update(src: &Path, dst: &Path) -> bool {
    if !dst.exists() {
        return true;
    }
    let src_meta = match fs::metadata(src) {
        Ok(m) => m,
        Err(_) => return true,
    };
    let dst_meta = match fs::metadata(dst) {
        Ok(m) => m,
        Err(_) => return true,
    };
    src_meta.modified().ok() > dst_meta.modified().ok()
}

fn create_backup(path: &Path) -> io::Result<()> {
    let backup_path = format!("{}.bak", path.display());
    fs::copy(path, &backup_path)?;
    Ok(())
}

fn prompt_overwrite(_src: &Path, dst: &Path) -> bool {
    print!("overwrite '{}' (y/n)? ", dst.display());
    let _ = io::stdout().flush();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().eq_ignore_ascii_case("y") || input.trim() == "yes"
}

fn copy_single_file(
    src: &Path,
    dst: &Path,
    opts: &Options,
    state: &mut CopyState,
) -> io::Result<()> {
    let src_canonical = src.canonicalize().map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("cannot canonicalize '{}': {}", src.display(), e),
        )
    })?;

    let dst_canonical = canonicalize_or_original(dst);

    if src_canonical == dst_canonical {
        return Err(io::Error::other(format!(
            "'{}' and '{}' are the same file",
            src.display(),
            dst.display()
        )));
    }

    if dst.exists() && opts.no_clobber && !opts.force {
        if !opts.quiet {
            eprintln!("{}: not overwriting '{}'", src.display(), dst.display());
        }
        state.skipped += 1;
        return Ok(());
    }

    if opts.update && !should_update(src, dst) {
        state.skipped += 1;
        if opts.verbose && !opts.quiet {
            println!("{}: {}", "skipped (not newer)".yellow(), src.display());
        }
        return Ok(());
    }

    if dst.exists() && opts.interactive && !prompt_overwrite(src, dst) {
        state.skipped += 1;
        if opts.verbose && !opts.quiet {
            println!("{}: {}", "skipped (user declined)".yellow(), src.display());
        }
        return Ok(());
    }

    if !dst.exists()
        && let Some(parent) = dst.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    if opts.dry_run {
        if opts.verbose || !opts.quiet {
            println!(
                "{} -> {} {}",
                src.display(),
                dst.display(),
                "[dry-run]".bright_black()
            );
        }
        state.files_copied += 1;
        return Ok(());
    }

    if dst.exists() && opts.backup {
        create_backup(dst)?;
    }

    let bytes = if opts.hard_link {
        std::fs::hard_link(src, dst).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("cannot create hard link '{}': {}", dst.display(), e),
            )
        })?;
        let src_meta = fs::metadata(src)?;
        src_meta.len()
    } else {
        utils::copy_file_with_progress(src, dst, opts.progress)?
    };

    if opts.preserve {
        utils::preserve_metadata(src, dst).ok();
    }

    state.bytes_copied += bytes;
    state.files_copied += 1;

    if opts.verbose && !opts.quiet {
        let link_tag = if opts.hard_link { " (hard link)" } else { "" };
        println!(
            "{} -> {}{}{}",
            src.display().green(),
            dst.display().cyan(),
            link_tag.bright_black(),
            if opts.backup && dst.exists() {
                " [backed up]".to_string()
            } else {
                String::new()
            }
        );
    }

    Ok(())
}

fn copy_symlink(src: &Path, dst: &Path, opts: &Options, state: &mut CopyState) -> io::Result<()> {
    let target = fs::read_link(src)?;

    if dst.exists() && opts.no_clobber && !opts.force {
        if !opts.quiet {
            eprintln!("{}: not overwriting '{}'", src.display(), dst.display());
        }
        state.skipped += 1;
        return Ok(());
    }

    if opts.dry_run {
        if opts.verbose || !opts.quiet {
            println!(
                "{} -> {} {} (symlink to {})",
                src.display(),
                dst.display(),
                "[dry-run]".bright_black(),
                target.display()
            );
        }
        state.files_copied += 1;
        return Ok(());
    }

    if !dst.exists()
        && let Some(parent) = dst.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    symlink_file(&target, dst)?;

    state.files_copied += 1;

    if opts.verbose && !opts.quiet {
        println!(
            "{} -> {} (symlink to {})",
            src.display().green(),
            dst.display().cyan(),
            target.display().bright_black()
        );
    }

    Ok(())
}

#[cfg(unix)]
fn symlink_file<T: AsRef<Path>, U: AsRef<Path>>(target: T, link: U) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn symlink_file<T: AsRef<Path>, U: AsRef<Path>>(target: T, link: U) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    original_src: &Path,
    opts: &Options,
    state: &mut CopyState,
    matcher: &Option<globset::GlobSet>,
) -> io::Result<()> {
    let src_canonical = src.canonicalize().map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("cannot canonicalize '{}': {}", src.display(), e),
        )
    })?;

    let effective_dst = if !opts.no_target_dir && dst.exists() && dst.is_dir() {
        let src_name = src
            .file_name()
            .ok_or_else(|| io::Error::other("cannot extract dirname"))?;
        dst.join(src_name)
    } else {
        dst.to_path_buf()
    };

    let dst_canonical = canonicalize_or_original(&effective_dst);

    if src_canonical == dst_canonical {
        return Err(io::Error::other(format!(
            "'{}' and '{}' are the same directory",
            original_src.display(),
            dst.display()
        )));
    }

    if dst.exists() && dst.is_file() {
        return Err(io::Error::other(format!(
            "cannot overwrite file '{}' with directory",
            dst.display()
        )));
    }

    if opts.progress {
        state.total_bytes = utils::count_dir_bytes(src, opts.follow_symlinks).unwrap_or(0);
    }

    let mut bar: Option<indicatif::ProgressBar> = None;
    if opts.progress && state.total_bytes > 0 {
        bar = Some(indicatif::ProgressBar::new(state.total_bytes));
        if let Some(b) = bar.as_mut() {
            b.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{spinner:.green} {percent}% [{elapsed_precise}] {bytes:>7}/{total_bytes:7} {bar:40.cyan/blue} {msg}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            b.set_message(format!("copying {}", original_src.display()));
        }
    }

    copy_dir_impl(src, &effective_dst, opts, state, &mut bar, matcher)?;

    if let Some(b) = bar {
        b.finish_and_clear();
    }

    Ok(())
}

fn copy_dir_impl(
    src: &Path,
    dst: &Path,
    opts: &Options,
    state: &mut CopyState,
    bar: &mut Option<indicatif::ProgressBar>,
    matcher: &Option<globset::GlobSet>,
) -> io::Result<()> {
    if opts.dry_run {
        if opts.verbose || !opts.quiet {
            println!(
                "{} -> {} {}",
                src.display().green(),
                dst.display().cyan(),
                "[dry-run]".bright_black()
            );
        }
    } else {
        fs::create_dir_all(dst)?;
    }

    state.dirs_copied += 1;

    if !opts.dry_run && opts.preserve {
        utils::preserve_metadata(src, dst).ok();
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_entry = entry.path();
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if let Some(matcher) = matcher
            && matcher.is_match(filename_str.as_ref())
        {
            state.skipped += 1;
            if opts.verbose && !opts.quiet {
                println!("{}: {}", "skipped".yellow(), filename_str);
            }
            continue;
        }

        let dst_entry = dst.join(&filename);

        let meta = if opts.follow_symlinks {
            fs::metadata(&src_entry)
        } else {
            fs::symlink_metadata(&src_entry)
        };

        let meta = match meta {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.file_type().is_symlink() && !opts.follow_symlinks {
            if opts.dry_run {
                if opts.verbose || !opts.quiet {
                    let target = fs::read_link(&src_entry).unwrap_or_default();
                    println!(
                        "{} -> {} {} (symlink to {})",
                        src_entry.display(),
                        dst_entry.display(),
                        "[dry-run]".bright_black(),
                        target.display()
                    );
                }
                state.files_copied += 1;
                continue;
            }

            let target = fs::read_link(&src_entry)?;
            symlink_file(&target, &dst_entry).ok();
            state.files_copied += 1;

            if opts.verbose && !opts.quiet {
                let target = fs::read_link(&src_entry).unwrap_or_default();
                println!(
                    "{} -> {} (symlink to {})",
                    src_entry.display().green(),
                    dst_entry.display().cyan(),
                    target.display().bright_black()
                );
            }
        } else if meta.is_dir() {
            copy_dir_impl(&src_entry, &dst_entry, opts, state, bar, matcher)?;
        } else {
            if dst_entry.exists() && opts.no_clobber && !opts.force {
                if !opts.quiet {
                    eprintln!(
                        "{}: not overwriting '{}'",
                        src_entry.display(),
                        dst_entry.display()
                    );
                }
                state.skipped += 1;
                continue;
            }

            if opts.update && !should_update(&src_entry, &dst_entry) {
                state.skipped += 1;
                if opts.verbose && !opts.quiet {
                    println!(
                        "{}: {}",
                        "skipped (not newer)".yellow(),
                        src_entry.display()
                    );
                }
                continue;
            }

            if dst_entry.exists() && opts.interactive && !prompt_overwrite(&src_entry, &dst_entry) {
                state.skipped += 1;
                if opts.verbose && !opts.quiet {
                    println!(
                        "{}: {}",
                        "skipped (user declined)".yellow(),
                        src_entry.display()
                    );
                }
                continue;
            }

            if opts.dry_run {
                if opts.verbose || !opts.quiet {
                    println!(
                        "{} -> {} {}",
                        src_entry.display(),
                        dst_entry.display(),
                        "[dry-run]".bright_black()
                    );
                }
                state.files_copied += 1;
                continue;
            }

            if dst_entry.exists() && opts.backup {
                create_backup(&dst_entry).ok();
            }

            let bytes = utils::copy_file_with_progress(&src_entry, &dst_entry, false)?;
            state.bytes_copied += bytes;
            state.files_copied += 1;

            if let Some(b) = bar.as_mut() {
                b.inc(bytes);
            }

            if opts.preserve {
                utils::preserve_metadata(&src_entry, &dst_entry).ok();
            }

            if opts.verbose && !opts.quiet {
                println!(
                    "{} -> {}",
                    src_entry.display().green(),
                    dst_entry.display().cyan()
                );
            }
        }
    }

    Ok(())
}
