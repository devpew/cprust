use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::cli::Options;
use crate::utils;

pub struct CopyState {
    pub bytes_copied: u64,
    pub files_copied: u64,
    pub total_bytes: u64,
}

impl CopyState {
    pub fn new() -> Self {
        Self {
            bytes_copied: 0,
            files_copied: 0,
            total_bytes: 0,
        }
    }
}

pub fn copy_sources(sources: &[PathBuf], destination: &Path, opts: &Options) -> io::Result<()> {
    let dst = utils::resolve_path(destination);
    let mut state = CopyState::new();

    if !dst.exists() {
        if sources.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "cannot copy multiple sources to '{}' which does not exist",
                    dst.display()
                ),
            ));
        }
        copy_one(&sources[0], &dst, opts, &mut state)?;
        return Ok(());
    }

    if !dst.is_dir() {
        if sources.len() > 1 {
            return Err(io::Error::other(format!(
                "cannot copy multiple sources to non-directory '{}'",
                dst.display()
            )));
        }
        if opts.no_clobber && !opts.force {
            if !opts.quiet {
                eprintln!(
                    "{}: not overwriting '{}'",
                    sources[0].display(),
                    dst.display()
                );
            }
            return Ok(());
        }
        copy_one(&sources[0], &dst, opts, &mut state)?;
        return Ok(());
    }

    for src in sources {
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

        copy_one(src, &final_dest, opts, &mut state)?;
    }

    Ok(())
}

fn copy_one(src: &Path, dst: &Path, opts: &Options, state: &mut CopyState) -> io::Result<()> {
    let src_resolved = utils::resolve_path(src);

    if !src_resolved.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("cannot stat '{}': No such file or directory", src.display()),
        ));
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
        return copy_dir_recursive(&src_resolved, dst, src, opts, state);
    }

    copy_single_file(&src_resolved, dst, opts, state)
}

fn canonicalize_or_original(p: &Path) -> PathBuf {
    if p.exists() {
        return p.canonicalize().ok().unwrap_or_else(|| p.to_path_buf());
    }
    // Find nearest existing ancestor
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
        return Ok(());
    }

    if !dst.exists()
        && let Some(parent) = dst.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let bytes = utils::copy_file_with_progress(src, dst, opts.progress)?;

    if opts.preserve {
        utils::preserve_metadata(src, dst)?;
    }

    state.bytes_copied += bytes;
    state.files_copied += 1;

    if opts.verbose && !opts.quiet {
        println!("{} -> {}", src.display(), dst.display());
    }

    Ok(())
}

fn copy_symlink(src: &Path, dst: &Path, opts: &Options, state: &mut CopyState) -> io::Result<()> {
    let target = fs::read_link(src)?;

    if dst.exists() && opts.no_clobber && !opts.force {
        if !opts.quiet {
            eprintln!("{}: not overwriting '{}'", src.display(), dst.display());
        }
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
            src.display(),
            dst.display(),
            target.display()
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
) -> io::Result<()> {
    let src_canonical = src.canonicalize().map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("cannot canonicalize '{}': {}", src.display(), e),
        )
    })?;

    let effective_dst = if dst.exists() && dst.is_dir() {
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

    copy_dir_impl(src, &effective_dst, opts, state, &mut bar)?;

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
) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    if opts.preserve {
        utils::preserve_metadata(src, dst).ok();
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_entry = entry.path();
        let dst_entry = dst.join(entry.file_name());

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
            let target = fs::read_link(&src_entry)?;
            symlink_file(&target, &dst_entry).ok();
            state.files_copied += 1;

            if opts.verbose && !opts.quiet {
                println!(
                    "{} -> {} (symlink to {})",
                    src_entry.display(),
                    dst_entry.display(),
                    target.display()
                );
            }
        } else if meta.is_dir() {
            copy_dir_impl(&src_entry, &dst_entry, opts, state, bar)?;
        } else {
            if dst_entry.exists() && opts.no_clobber && !opts.force {
                if !opts.quiet {
                    eprintln!(
                        "{}: not overwriting '{}'",
                        src_entry.display(),
                        dst_entry.display()
                    );
                }
                continue;
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
                println!("{} -> {}", src_entry.display(), dst_entry.display());
            }
        }
    }

    Ok(())
}
