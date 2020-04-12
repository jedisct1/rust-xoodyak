use criterion::{criterion_group, criterion_main, Criterion};
use xoodyak::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Xoodoo permutation", |b| {
        let mut out = [0u8; 48];
        let mut st = Xoodoo::default();
        b.iter(|| {
            st.permute();
            st.bytes(&mut out);
            out
        })
    });

    c.bench_function("Xoodyak hash", |b| {
        let mut out = [0u8; 64];
        let mut st = XoodyakHash::new();
        b.iter(|| {
            st.absorb(b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. ");
            st.squeeze(&mut out);
            out
        })
    });

    c.bench_function("Xoodyak keyed", |b| {
        let mut out = [0u8; 64];
        let mut st = XoodyakKeyed::new(b"key", None,None, None).unwrap();
        b.iter(|| {
            st.absorb(b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. ");
            st.squeeze(&mut out);
            out
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
