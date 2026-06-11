use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs;
use std::io::Write;
use std::process::Command;

fn bin_path() -> String {
    format!("{}/target/release/cprust", env!("CARGO_MANIFEST_DIR"))
}

fn create_test_file(dir: &str, name: &str, size_kb: usize) {
    let path = format!("{}/{}", dir, name);
    fs::create_dir_all(dir).unwrap();
    let data = vec![b'A'; size_kb * 1024];
    fs::write(&path, &data).unwrap();
}

fn bench_copy_single_file(c: &mut Criterion) {
    let src_dir = "/tmp/cprust_bench_src";
    let dst = "/tmp/cprust_bench_dst";
    let _ = fs::remove_dir_all(src_dir);
    let _ = fs::remove_file(dst);

    create_test_file(src_dir, "test.bin", 1024);

    c.bench_function("copy 1MB file", |b| {
        b.iter(|| {
            let _ = fs::remove_file(dst);
            Command::new(&bin_path())
                .args(["test.bin", dst])
                .current_dir(src_dir)
                .output()
                .unwrap();
        })
    });

    let _ = fs::remove_dir_all(src_dir);
    let _ = fs::remove_file(dst);
}

fn bench_copy_directory(c: &mut Criterion) {
    let src_dir = "/tmp/cprust_bench_dir_src";
    let dst_dir = "/tmp/cprust_bench_dir_dst";
    let _ = fs::remove_dir_all(src_dir);
    let _ = fs::remove_dir_all(dst_dir);

    for i in 0..10 {
        create_test_file(
            &format!("{}/sub{}", src_dir, i / 5),
            &format!("file{}.bin", i),
            100,
        );
    }

    c.bench_function("copy dir (10 files, 1MB total)", |b| {
        b.iter(|| {
            let _ = fs::remove_dir_all(dst_dir);
            Command::new(&bin_path())
                .args(["-r", ".", dst_dir])
                .current_dir(src_dir)
                .output()
                .unwrap();
        })
    });

    let _ = fs::remove_dir_all(src_dir);
    let _ = fs::remove_dir_all(dst_dir);
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_copy_single_file(c);
    bench_copy_directory(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
