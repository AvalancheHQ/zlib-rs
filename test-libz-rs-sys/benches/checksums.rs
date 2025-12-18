use codspeed_criterion_compat::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};

fn bench_crc32(c: &mut Criterion) {
    let silesia_small_tar = include_bytes!("../../silesia-small.tar");

    let mut group = c.benchmark_group("crc32");
    group.throughput(Throughput::Bytes(silesia_small_tar.len() as u64));

    group.bench_function("full", |b| {
        b.iter(|| {
            let mut state = zlib_rs::crc32::Crc32Fold::new();
            state.fold(silesia_small_tar, 0);
            state.finish()
        });
    });

    for chunk_size in [32, 1024, 4096] {
        group.bench_with_input(
            BenchmarkId::new("chunked", chunk_size),
            &chunk_size,
            |b, &chunk_size| {
                b.iter(|| {
                    let mut state = zlib_rs::crc32::Crc32Fold::new();
                    for chunk in silesia_small_tar.chunks(chunk_size) {
                        state.fold(chunk, 0);
                    }
                    state.finish()
                });
            },
        );
    }
    group.finish();
}

fn bench_adler32(c: &mut Criterion) {
    let silesia_small_tar = include_bytes!("../../silesia-small.tar");

    let mut group = c.benchmark_group("adler32");
    group.throughput(Throughput::Bytes(silesia_small_tar.len() as u64));

    group.bench_function("full", |b| {
        b.iter(|| zlib_rs::adler32(42, silesia_small_tar));
    });

    for chunk_size in [1024, 4096, 16384] {
        group.bench_with_input(
            BenchmarkId::new("chunked", chunk_size),
            &chunk_size,
            |b, &chunk_size| {
                b.iter(|| {
                    let mut adler = unsafe { libz_rs_sys::adler32(0, std::ptr::null(), 0) };
                    for chunk in silesia_small_tar.chunks(chunk_size) {
                        adler = unsafe {
                            libz_rs_sys::adler32(adler, chunk.as_ptr(), chunk.len() as _)
                        };
                    }
                    adler
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_crc32, bench_adler32);
criterion_main!(benches);
