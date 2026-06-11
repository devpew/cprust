use std::fs;
use std::io::Write;
use std::process::Command;

fn bin_path() -> String {
    format!("{}/target/debug/cprust", env!("CARGO_MANIFEST_DIR"))
}

fn run_cp(args: &[&str]) -> Command {
    let mut cmd = Command::new(bin_path());
    cmd.args(args);
    cmd
}

fn create_file(dir: &str, name: &str, content: &str) -> String {
    let path = format!("{}/{}", dir, name);
    fs::create_dir_all(dir).unwrap();
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path
}

fn cleanup(path: &str) {
    if path.starts_with("/tmp/cprust_test") {
        let _ = fs::remove_dir_all(path);
    }
}

#[test]
fn test_copy_single_file() {
    let src_dir = "/tmp/cprust_test_src1";
    let dst = "/tmp/cprust_test_dst1";
    cleanup(src_dir);
    cleanup(dst);

    create_file(src_dir, "hello.txt", "hello world");

    let output = run_cp(&["hello.txt", dst])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dst.parse::<std::path::PathBuf>().unwrap().exists());

    cleanup(src_dir);
    cleanup(dst);
}

#[test]
fn test_copy_file_to_dir() {
    let src_dir = "/tmp/cprust_test_src2";
    let dst_dir = "/tmp/cprust_test_dst2";
    cleanup(src_dir);
    cleanup(dst_dir);

    create_file(src_dir, "file.txt", "content");
    fs::create_dir_all(dst_dir).unwrap();

    let output = run_cp(&["file.txt", dst_dir])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        format!("{}/file.txt", dst_dir)
            .parse::<std::path::PathBuf>()
            .unwrap()
            .exists()
    );

    cleanup(src_dir);
    cleanup(dst_dir);
}

#[test]
fn test_copy_missing_source() {
    let output = run_cp(&["nonexistent_file.txt", "/tmp/out.txt"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such file or directory"));
}

#[test]
fn test_copy_same_file() {
    let dir = "/tmp/cprust_test_same";
    cleanup(dir);

    create_file(dir, "same.txt", "data");

    let output = run_cp(&["same.txt", "same.txt"])
        .current_dir(dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("same file"));

    cleanup(dir);
}

#[test]
fn test_copy_directory_without_recursive() {
    let src_dir = "/tmp/cprust_test_norec_src";
    let dst = "/tmp/cprust_test_norec_dst";
    cleanup(src_dir);
    cleanup(dst);

    fs::create_dir_all(src_dir).unwrap();
    create_file(src_dir, "inner.txt", "inner");

    let output = run_cp(&[".", dst]).current_dir(src_dir).output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-r"));

    cleanup(src_dir);
    cleanup(dst);
}

#[test]
fn test_copy_directory_recursive() {
    let src_dir = "/tmp/cprust_test_rec_src";
    let dst = "/tmp/cprust_test_rec_dst";
    cleanup(src_dir);
    cleanup(dst);

    fs::create_dir_all(format!("{}/sub", src_dir)).unwrap();
    create_file(src_dir, "root.txt", "root content");
    create_file(src_dir, "sub/nested.txt", "nested content");

    let output = run_cp(&["-r", ".", dst])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        format!("{}/root.txt", dst)
            .parse::<std::path::PathBuf>()
            .unwrap()
            .exists()
    );
    assert!(
        format!("{}/sub/nested.txt", dst)
            .parse::<std::path::PathBuf>()
            .unwrap()
            .exists()
    );

    let content = fs::read_to_string(format!("{}/sub/nested.txt", dst)).unwrap();
    assert_eq!(content, "nested content");

    cleanup(src_dir);
    cleanup(dst);
}

#[test]
fn test_verbose_mode() {
    let src_dir = "/tmp/cprust_test_verbose_src";
    let dst = "/tmp/cprust_test_verbose_dst";
    cleanup(src_dir);
    cleanup(dst);

    create_file(src_dir, "vfile.txt", "verbose test");

    let output = run_cp(&["-v", "vfile.txt", dst])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vfile.txt"));

    cleanup(src_dir);
    cleanup(dst);
}

#[test]
fn test_no_clobber() {
    let src_dir = "/tmp/cprust_test_nc_src";
    let dst_dir = "/tmp/cprust_test_nc_dst";
    let dst_file = "/tmp/cprust_test_nc_dst/file.txt";
    cleanup(src_dir);
    cleanup(dst_dir);

    create_file(src_dir, "file.txt", "source");
    create_file(dst_dir, "file.txt", "existing");

    let output = run_cp(&["-n", "file.txt", dst_file])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let content = fs::read_to_string(dst_file).unwrap();
    assert_eq!(content, "existing");

    cleanup(src_dir);
    cleanup(dst_dir);
}

#[test]
fn test_force_overwrite() {
    let src_dir = "/tmp/cprust_test_force_src";
    let dst_dir = "/tmp/cprust_test_force_dst";
    let dst_file = "/tmp/cprust_test_force_dst/file.txt";
    cleanup(src_dir);
    cleanup(dst_dir);

    create_file(src_dir, "file.txt", "new content");
    create_file(dst_dir, "file.txt", "old content");

    let output = run_cp(&["-f", "file.txt", dst_file])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let content = fs::read_to_string(dst_file).unwrap();
    assert_eq!(content, "new content");

    cleanup(src_dir);
    cleanup(dst_dir);
}

#[test]
fn test_multiple_sources_to_dir() {
    let src_dir = "/tmp/cprust_test_multi_src";
    let dst_dir = "/tmp/cprust_test_multi_dst";
    cleanup(src_dir);
    cleanup(dst_dir);

    fs::create_dir_all(dst_dir).unwrap();
    create_file(src_dir, "a.txt", "aaa");
    create_file(src_dir, "b.txt", "bbb");

    let output = run_cp(&["a.txt", "b.txt", dst_dir])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        format!("{}/a.txt", dst_dir)
            .parse::<std::path::PathBuf>()
            .unwrap()
            .exists()
    );
    assert!(
        format!("{}/b.txt", dst_dir)
            .parse::<std::path::PathBuf>()
            .unwrap()
            .exists()
    );

    cleanup(src_dir);
    cleanup(dst_dir);
}

#[test]
fn test_multiple_sources_non_dir_dest() {
    let src_dir = "/tmp/cprust_test_multi2_src";
    let dst = "/tmp/cprust_test_multi2_dst";
    cleanup(src_dir);
    cleanup(dst);

    create_file(src_dir, "a.txt", "aaa");
    create_file(src_dir, "b.txt", "bbb");

    let output = run_cp(&["a.txt", "b.txt", dst])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));

    cleanup(src_dir);
}

#[test]
fn test_relative_paths() {
    let workdir = "/tmp/cprust_test_rel";
    cleanup(workdir);

    fs::create_dir_all(format!("{}/src", workdir)).unwrap();
    fs::create_dir_all(format!("{}/dst", workdir)).unwrap();
    create_file(workdir, "src/data.txt", "relative test");

    let output = run_cp(&["src/data.txt", "dst/"])
        .current_dir(workdir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        format!("{}/dst/data.txt", workdir)
            .parse::<std::path::PathBuf>()
            .unwrap()
            .exists()
    );

    cleanup(workdir);
}

#[test]
fn test_symlink_copy() {
    let dir = "/tmp/cprust_test_symlink";
    cleanup(dir);

    fs::create_dir_all(dir).unwrap();
    create_file(dir, "real.txt", "real file");

    #[cfg(unix)]
    {
        use std::os::unix::fs as unix_fs;
        let real_path = format!("{}/real.txt", dir);
        let link_path = format!("{}/link.txt", dir);
        unix_fs::symlink(&real_path, &link_path).unwrap();
    }

    let dst = format!("{}/copied", dir);

    let output = run_cp(&["link.txt", &dst])
        .current_dir(dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    cleanup(dir);
}

#[test]
fn test_help_flag() {
    let output = run_cp(&["--help"]).output().unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage"));
}

#[test]
fn test_unknown_option() {
    let output = run_cp(&["--unknown", "a.txt", "b.txt"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown option"));
}

#[test]
fn test_missing_operands() {
    let output = run_cp(&["only_one"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing file operand"));
}

#[test]
fn test_preserve_flag() {
    let src_dir = "/tmp/cprust_test_preserve_src";
    let dst = "/tmp/cprust_test_preserve_dst";
    cleanup(src_dir);
    cleanup(dst);

    create_file(src_dir, "preserve.txt", "preserve me");

    let output = run_cp(&["-p", "preserve.txt", dst])
        .current_dir(src_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dst.parse::<std::path::PathBuf>().unwrap().exists());

    cleanup(src_dir);
    cleanup(dst);
}
