use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use std::path::PathBuf;

fn get_test_data_path(filename: &str) -> PathBuf {
    // Try to find the file in the workspace root
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push(filename);
    
    if path.exists() {
        return path;
    }
    
    // Fallback: try current directory
    let path = PathBuf::from(filename);
    if path.exists() {
        return path;
    }
    
    panic!("Could not find test data file: {}", filename);
}

fn compress_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress");
    
    // Read test data
    let test_data_path = get_test_data_path("silesia-small.tar");
    let test_data = fs::read(&test_data_path)
        .expect("Failed to read test data");
    
    // Benchmark different compression levels
    for level in [1, 6, 9] {
        group.bench_with_input(
            BenchmarkId::new("deflate", level),
            &level,
            |b, &level| {
                b.iter(|| {
                    let mut compressed = Vec::new();
                    let mut encoder = flate2::write::DeflateEncoder::new(
                        &mut compressed,
                        flate2::Compression::new(level),
                    );
                    std::io::Write::write_all(&mut encoder, &test_data).unwrap();
                    encoder.finish().unwrap();
                    compressed
                });
            },
        );
    }
    
    group.finish();
}

fn decompress_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress");

    // Read compressed test data
    let compressed_path = get_test_data_path("silesia-small.tar.gz");
    let compressed_data = fs::read(&compressed_path)
        .expect("Failed to read compressed test data");
    
    group.bench_function("inflate", |b| {
        b.iter(|| {
            let mut decompressed = Vec::new();
            let mut decoder = flate2::read::GzDecoder::new(&compressed_data[..]);
            std::io::Read::read_to_end(&mut decoder, &mut decompressed).unwrap();
            decompressed
        });
    });
    
    group.finish();
}

fn crc32_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("crc32");
    
    let test_data_path = get_test_data_path("silesia-small.tar");
    let test_data = fs::read(&test_data_path)
        .expect("Failed to read test data");
    
    group.bench_function("crc32", |b| {
        b.iter(|| {
            zlib_rs::crc32(0, &test_data)
        });
    });
    
    group.finish();
}

fn adler32_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("adler32");
    
    let test_data_path = get_test_data_path("silesia-small.tar");
    let test_data = fs::read(&test_data_path)
        .expect("Failed to read test data");
    
    group.bench_function("adler32", |b| {
        b.iter(|| {
            zlib_rs::adler32(1, &test_data)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    compress_benchmarks,
    decompress_benchmarks,
    crc32_benchmarks,
    adler32_benchmarks
);
criterion_main!(benches);
