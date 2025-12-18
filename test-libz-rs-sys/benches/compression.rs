use codspeed_criterion_compat::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::ffi::{c_int, c_uint};
use zlib_rs::{DeflateFlush, ReturnCode};

const METHOD: i32 = zlib_rs::c_api::Z_DEFLATED;
const WINDOW_BITS: i32 = 15;
const MEM_LEVEL: i32 = 8;
const STRATEGY: i32 = zlib_rs::c_api::Z_DEFAULT_STRATEGY;

fn compress_rs(dest: &mut [u8], dest_len: &mut usize, source: &[u8], level: i32) -> ReturnCode {
    use libz_rs_sys::{deflate, deflateEnd, deflateInit2_, z_stream, zlibVersion};

    let mut stream = z_stream {
        next_in: source.as_ptr() as *mut u8,
        avail_in: 0,
        total_in: 0,
        next_out: dest.as_mut_ptr(),
        avail_out: 0,
        total_out: 0,
        msg: std::ptr::null_mut(),
        state: std::ptr::null_mut(),
        zalloc: Some(zlib_rs::allocate::C.zalloc),
        zfree: Some(zlib_rs::allocate::C.zfree),
        opaque: std::ptr::null_mut(),
        data_type: 0,
        adler: 0,
        reserved: 0,
    };

    let err = {
        let strm: *mut z_stream = &mut stream;
        unsafe {
            deflateInit2_(
                strm,
                level,
                METHOD,
                WINDOW_BITS,
                MEM_LEVEL,
                STRATEGY,
                zlibVersion(),
                std::mem::size_of::<z_stream>() as c_int,
            )
        }
    };

    if ReturnCode::from(err) != ReturnCode::Ok {
        return ReturnCode::from(err);
    }

    let max = c_uint::MAX as usize;
    let mut left = dest.len();
    let mut source_len = source.len();

    loop {
        if stream.avail_out == 0 {
            stream.avail_out = Ord::min(left, max) as _;
            left -= stream.avail_out as usize;
        }

        if stream.avail_in == 0 {
            stream.avail_in = Ord::min(source_len, max) as _;
            source_len -= stream.avail_in as usize;
        }

        let flush = if source_len > 0 {
            DeflateFlush::NoFlush
        } else {
            DeflateFlush::Finish
        };

        let err = unsafe { deflate(&mut stream, flush as i32) };
        if ReturnCode::from(err) != ReturnCode::Ok {
            break;
        }
    }

    *dest_len = stream.total_out as _;
    unsafe { deflateEnd(&mut stream) };
    ReturnCode::Ok
}

fn uncompress_rs(dest: &mut [u8], dest_len: &mut usize, source: &[u8]) -> ReturnCode {
    let source_len = source.len() as _;
    let err = unsafe {
        libz_rs_sys::uncompress(
            dest.as_mut_ptr(),
            dest_len as *mut _ as *mut _,
            source.as_ptr(),
            source_len,
        )
    };
    ReturnCode::from(err)
}

fn bench_compress(c: &mut Criterion) {
    let silesia_small_tar = include_bytes!("../../silesia-small.tar");
    let mut dest_vec = vec![0u8; 1 << 28];

    let mut group = c.benchmark_group("compress");
    group.throughput(Throughput::Bytes(silesia_small_tar.len() as u64));

    for level in [1, 6, 9] {
        group.bench_with_input(BenchmarkId::new("level", level), &level, |b, &level| {
            b.iter(|| {
                let mut dest_len = dest_vec.len();
                let result = compress_rs(&mut dest_vec, &mut dest_len, silesia_small_tar, level);
                assert_eq!(result, ReturnCode::Ok);
                dest_len
            });
        });
    }
    group.finish();
}

fn bench_uncompress(c: &mut Criterion) {
    let silesia_small_tar_gz = include_bytes!("../../silesia-small.tar.gz");
    let mut dest_vec = vec![0u8; 1 << 28];

    let mut group = c.benchmark_group("uncompress");
    group.throughput(Throughput::Bytes(silesia_small_tar_gz.len() as u64));

    group.bench_function("decompress", |b| {
        b.iter(|| {
            let mut dest_len = dest_vec.len();
            let result = uncompress_rs(&mut dest_vec, &mut dest_len, silesia_small_tar_gz);
            assert_eq!(result, ReturnCode::Ok);
            dest_len
        });
    });
    group.finish();
}

criterion_group!(benches, bench_compress, bench_uncompress);
criterion_main!(benches);
