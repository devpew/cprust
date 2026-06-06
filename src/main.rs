use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_entry = entry.path();
        let dst_entry = dst.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir(&src_entry, &dst_entry)?;
        } else {
            fs::copy(&src_entry, &dst_entry)?;
        }
    }

    Ok(())
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut non_flags: Vec<String> = args.iter().skip(1).filter(|a| !a.starts_with('-')).cloned().collect();

    if non_flags.len() < 2 {
        eprintln!("Usage: {} [-r] <source> <destination>", args[0]);
        std::process::exit(1);
    }

    let destination = non_flags.pop().unwrap();
    let source = non_flags.remove(0);

    let cwd = env::current_dir()?;
    let src_path = if Path::new(&source).is_absolute() {
        source.clone()
    } else {
        cwd.join(&source).into_os_string().into_string().unwrap_or(source.clone())
    };
    let src_path = Path::new(&src_path);
    if !src_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("cannot stat '{}': No such file or directory", source),
        ));
    }

    let dest_resolved = if Path::new(&destination).is_absolute() {
        destination.clone()
    } else {
        cwd.join(&destination).into_os_string().into_string().unwrap_or(destination.clone())
    };
    let dst_path = Path::new(&dest_resolved);

    let src_canonical = src_path.canonicalize()?;

    if src_path.is_dir() {
        let final_dest = if dst_path.exists() && dst_path.is_dir() {
            let src_filename = src_path
                .file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot extract dirname"))?;
            dst_path.join(src_filename)
        } else {
            dst_path.to_path_buf()
        };

        let dest_canonical = if final_dest.exists() {
            final_dest.canonicalize()?
        } else if let Some(parent) = final_dest.parent() {
            parent.canonicalize()?.join(final_dest.file_name().unwrap_or_default())
        } else {
            final_dest.clone()
        };

        if src_canonical == dest_canonical {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("'{}' and '{}' are the same directory", source, destination),
            ));
        }

        if dst_path.exists() && dst_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("cannot overwrite file '{}' with directory", destination),
            ));
        }

        copy_dir(src_path, &final_dest)?;
        return Ok(());
    }

    let final_dest = if dst_path.exists() && dst_path.is_dir() {
        let src_filename = src_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot extract filename"))?;
        dst_path.join(src_filename)
    } else {
        dst_path.to_path_buf()
    };

    let dest_canonical = if final_dest.exists() {
        final_dest.canonicalize()?
    } else if let (Some(parent), Some(filename)) = (final_dest.parent(), final_dest.file_name()) {
        parent.canonicalize()?.join(filename)
    } else {
        final_dest.clone()
    };

    if src_canonical == dest_canonical {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("'{}' and '{}' are the same file", source, destination),
        ));
    }

    fs::copy(src_path, &final_dest)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", env::args().next().unwrap_or_else(|| "cp".into()), e);
        std::process::exit(1);
    }
}
