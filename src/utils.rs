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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_resolve_absolute_path() {
        let p = Path::new("/tmp/test");
        let resolved = resolve_path(p);
        assert!(resolved.is_absolute());
        assert_eq!(resolved, Path::new("/tmp/test"));
    }

    #[test]
    fn test_resolve_relative_path() {
        let cwd = env::current_dir().unwrap();
        let p = Path::new("foo/bar");
        let resolved = resolve_path(p);
        assert_eq!(resolved, cwd.join("foo/bar"));
    }

    #[test]
    fn test_build_parents_path() {
        let src = Path::new("a/b/c/file.txt");
        let dst = Path::new("/backup");
        let result = build_parents_path(src, dst);
        assert_eq!(result, Path::new("/backup/a/b/c"));
    }

    #[test]
    fn test_build_parents_path_no_parent() {
        let src = Path::new("file.txt");
        let dst = Path::new("/backup");
        let result = build_parents_path(src, dst);
        assert_eq!(result, Path::new("/backup"));
    }

    #[test]
    fn test_count_dir_bytes() {
        let dir = "/tmp/cprust_unit_count";
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(format!("{}/sub", dir)).unwrap();

        let mut f1 = fs::File::create(format!("{}/a.txt", dir)).unwrap();
        f1.write_all(b"12345").unwrap();

        let mut f2 = fs::File::create(format!("{}/sub/b.txt", dir)).unwrap();
        f2.write_all(b"6789012345").unwrap();

        let total = count_dir_bytes(Path::new(dir), false).unwrap();
        assert_eq!(total, 15);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_count_empty_dir_bytes() {
        let dir = "/tmp/cprust_unit_empty";
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir).unwrap();

        let total = count_dir_bytes(Path::new(dir), false).unwrap();
        assert_eq!(total, 0);

        let _ = fs::remove_dir_all(dir);
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
