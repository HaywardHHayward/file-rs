use std::{
    cmp::max,
    ffi::OsString,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use file::classify_file;
use itertools::Itertools;

const FILE_LIST: [&str; 15] = [
    "./test_files/ascii.txt", // ascii 0-1
    "./test_files/harpers_ASCII.txt",
    "./test_files/die_ISO-8859-1.txt", // latin1 2-4
    "./test_files/iso8859-1.txt",
    "./test_files/portugal_ISO-8859-1.txt",
    "./test_files/shisei_UTF-8.txt", // utf8 5-7
    "./test_files/utf8.txt",
    "./test_files/utf8_test.txt",
    "./test_files/be_utf16.txt", // utf16 8-11
    "./test_files/le_utf16.txt",
    "./test_files/shisei_UTF-16BE.txt",
    "./test_files/shisei_UTF-16LE.txt",
    "./test_files/gb.txt", // gb 12-13
    "./test_files/gb_test.txt",
    "./test_files/data.data", // data 14
];

const SMALL_LIST: [&str; 5] = [
    FILE_LIST[1],
    FILE_LIST[2],
    FILE_LIST[5],
    FILE_LIST[10],
    FILE_LIST[13],
];

fn file_length(path: &Path) -> u64 {
    std::fs::metadata(path).unwrap().len()
}

fn classification_bench(c: &mut Criterion) {
    let group_collection = FILE_LIST.iter().map(OsString::from).map(PathBuf::from);
    let mut group = c.benchmark_group("Classification");
    for path in group_collection.clone() {
        let bytes = file_length(&path);
        if bytes > 1024 * 500 {
            group.sample_size(25);
        } else if bytes > 1024 * 250 {
            group.sample_size(50);
        } else if bytes > 1024 * 100 {
            group.sample_size(75);
        } else if bytes > 1024 * 50 {
            group.sample_size(125);
        } else if bytes > 1024 * 10 {
            group.sample_size(250);
        } else {
            group.sample_size(500);
        }
        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::from_parameter(path.file_name().unwrap().to_string_lossy()),
            &path,
            |b, path| b.iter(|| classify_file(BufReader::new(File::open(path).unwrap()))),
        );
    }
    group.finish();
}

fn program_working_bench(c: &mut Criterion) {
    let group_collection = SMALL_LIST.iter().map(OsString::from);
    let mut group = c.benchmark_group("Program");
    group.sampling_mode(SamplingMode::Flat);
    for paths in group_collection.clone().powerset() {
        let max_bytes = paths
            .iter()
            .fold(0, |acc, path| max(file_length(Path::new(path)), acc));
        group.throughput(Throughput::Bytes(max_bytes));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!(
                "{} - {}",
                max_bytes,
                paths
                    .iter()
                    .map(|a| Path::new(a)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned())
                    .join("/")
            )),
            &paths,
            |b, paths| b.iter(|| file::file(paths.iter().cloned())),
        );
    }
    group.finish();
}

criterion_group!(benches, classification_bench, program_working_bench);

criterion_main!(benches);
