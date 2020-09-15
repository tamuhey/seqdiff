use criterion::{black_box, criterion_group, criterion_main, Criterion};
use seqdiff;

fn slow_ratio<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> f64 {
    let l = a.len() + b.len();
    if l == 0 {
        return 100.;
    }
    let (a2b, _) = seqdiff::diff_by(a, b, <A as PartialEq<B>>::eq);
    let m = a2b.iter().filter(|x| x.is_some()).count() * 2;
    ((100 * m) as f64) / (l as f64)
}

fn bench_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("Ratios");
    let s = black_box("1 1 1 1 1 1 1 1 1 1".chars().collect::<Vec<_>>());
    let t = black_box("1 2 1 1 2 1 1 4 1 1".chars().collect::<Vec<_>>());
    group.bench_function("slow", |b| b.iter(|| slow_ratio(&s, &t)));
    group.bench_function("normal", |b| b.iter(|| seqdiff::ratio(&s, &t)));
    group.finish();
}

criterion_group!(benches, bench_ratios);
criterion_main!(benches);
