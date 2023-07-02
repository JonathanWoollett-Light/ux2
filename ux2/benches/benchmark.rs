use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");

    let mut rng = rand::thread_rng();

    // Reduce warmup and measurement time so the benchmarks don't take as long.
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_millis(1000));

    #[cfg(feature = "8")]
    group.bench_with_input(BenchmarkId::new("core", 8), &(), |b, _| {
        let lhs = rng.gen_range(0..i8::MAX / 4);
        let rhs = rng.gen_range(0..i8::MAX / 4);
        b.iter(|| lhs + rhs)
    });
    #[cfg(feature = "16")]
    group.bench_with_input(BenchmarkId::new("core", 16), &(), |b, _| {
        let lhs = rng.gen_range(0..i16::MAX / 4);
        let rhs = rng.gen_range(0..i16::MAX / 4);
        b.iter(|| lhs + rhs)
    });
    #[cfg(feature = "32")]
    group.bench_with_input(BenchmarkId::new("core", 32), &(), |b, _| {
        let lhs = rng.gen_range(0..i32::MAX / 4);
        let rhs = rng.gen_range(0..i32::MAX / 4);
        b.iter(|| lhs + rhs)
    });
    #[cfg(feature = "64")]
    group.bench_with_input(BenchmarkId::new("core", 64), &(), |b, _| {
        let lhs = rng.gen_range(0..i64::MAX / 4);
        let rhs = rng.gen_range(0..i64::MAX / 4);
        b.iter(|| lhs + rhs)
    });

    #[cfg(feature = "8")]
    group.bench_with_input(BenchmarkId::new("ux2", 8), &(), |b, _| {
        let lhs = ux2::i7::try_from(rng.gen_range(0..i8::MAX / 4)).unwrap();
        let rhs = ux2::i7::try_from(rng.gen_range(0..i8::MAX / 4)).unwrap();
        b.iter(|| lhs + rhs)
    });
    #[cfg(feature = "16")]
    group.bench_with_input(BenchmarkId::new("ux2", 16), &(), |b, _| {
        let lhs = ux2::i15::from(rng.gen_range(0..i16::MAX / 4)).unwrap();
        let rhs = ux2::i15::from(rng.gen_range(0..i16::MAX / 4)).unwrap();
        b.iter(|| lhs + rhs)
    });
    #[cfg(feature = "32")]
    group.bench_with_input(BenchmarkId::new("ux2", 32), &(), |b, _| {
        let lhs = ux2::i31::from(rng.gen_range(0..i32::MAX / 4)).unwrap();
        let rhs = ux2::i31::from(rng.gen_range(0..i32::MAX / 4)).unwrap();
        b.iter(|| lhs + rhs)
    });
    #[cfg(any(feature = "64"))]
    group.bench_with_input(BenchmarkId::new("ux2", 64), &(), |b, _| {
        let lhs = ux2::i63::try_from(rng.gen_range(0..i64::MAX / 4)).unwrap();
        let rhs = ux2::i63::try_from(rng.gen_range(0..i64::MAX / 4)).unwrap();
        b.iter(|| lhs + rhs)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
