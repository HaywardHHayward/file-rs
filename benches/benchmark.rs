use std::{ffi::OsString, ops::RangeInclusive};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const FILE_LIST: [&str; 13] = [
    "./test_files/ascii.txt", // ascii 0-1
    "./test_files/harpers_ASCII.txt",
    "./test_files/die_ISO-8859-1.txt", // latin1 2-4
    "./test_files/iso8859-1.txt",
    "./test_files/portugal_ISO-8859-1.txt",
    "./test_files/shisei_UTF-8.txt", // utf8 5-7
    "./test_files/utf8.txt",
    "./test_files/utf8_test.txt",
    "./test_files/be_utf16.txt", // utf16 8-9
    "./test_files/le_utf16.txt",
    "./test_files/gb.txt", // gb 10-11
    "./test_files/gb_test.txt",
    "./test_files/data.data", // data 12
];

fn file_length(path: &OsString) -> u64 {
    std::fs::metadata(path).unwrap().len()
}

fn test_files(c: &mut Criterion, range_inclusive: RangeInclusive<usize>, group_name: String) {
    let group_collection = FILE_LIST[range_inclusive].iter().map(OsString::from);
    let mut group = c.benchmark_group(&group_name);
    for path in group_collection.clone() {
        group.throughput(Throughput::Bytes(file_length(&path)));
        group.bench_with_input(
            BenchmarkId::from_parameter(path.to_string_lossy()),
            &path,
            |b, path| b.iter(|| file::file(vec![path.to_owned()].into_iter())),
        );
    }
    group.finish();
}
fn all(c: &mut Criterion) {
    test_files(c, 0..=12, String::from("files"));
    let all = FILE_LIST.iter().map(OsString::from);
    c.bench_with_input(
        BenchmarkId::new("all files", "all files"),
        &all,
        |b, path| b.iter(|| file::file(path.to_owned())),
    );
}

criterion_group!(benches, all);

criterion_main!(benches);
