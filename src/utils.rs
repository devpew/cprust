use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn resolve_path(p: &Path) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();

    if p.is_absolute() {
        p.to_path_buf()
    } else {
        cwd.join(p)
    }
}

pub fn preserve_metadata(src: &Path, dst: &Path) -> io::Result<()> {
    let meta = fs::metadata(src)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        fs::set_permissions(dst, fs::Permissions::from_mode(mode))?;
    }

    let atime = filetime::FileTime::from_last_access_time(&meta);
    let mtime = filetime::FileTime::from_last_modification_time(&meta);
    filetime::set_file_times(dst, atime, mtime)?;

    Ok(())
}

pub fn copy_file_with_progress(src: &Path, dst: &Path, progress: bool) -> io::Result<u64> {
    let mut src_file = fs::File::open(src)?;
    let src_meta = src_file.metadata()?;
    let total = src_meta.len();

    let mut dst_file = fs::File::create(dst)?;

    if progress && total > 1024 * 1024 {
        let bar = indicatif::ProgressBar::new(total);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} {percent}% [{elapsed_precise}] {bytes:>7}/{total_bytes:7} {bar:40.cyan/blue} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        let mut buffer = vec![0u8; 64 * 1024];
        loop {
            let bytes_read = src_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            dst_file.write_all(&buffer[..bytes_read])?;
            bar.inc(bytes_read as u64);
        }

        bar.finish_and_clear();
    } else {
        std::io::copy(&mut src_file, &mut dst_file)?;
    }

    Ok(total)
}

pub fn count_dir_bytes(dir: &Path, follow_symlinks: bool) -> io::Result<u64> {
    let mut total: u64 = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let meta = if follow_symlinks {
            fs::metadata(&path)
        } else {
            fs::symlink_metadata(&path)
        };

        match meta {
            Ok(m) => {
                if m.is_dir() {
                    total += count_dir_bytes(&path, follow_symlinks)?;
                } else {
                    total += m.len();
                }
            }
            Err(_) => continue,
        }
    }

    Ok(total)
}

pub fn build_parents_path(src: &Path, dst: &Path) -> PathBuf {
    if let Some(parent) = src.parent() {
        dst.join(parent)
    } else {
        dst.to_path_buf()
    }
}
