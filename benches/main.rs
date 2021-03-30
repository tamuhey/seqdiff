use criterion::{black_box, criterion_group, criterion_main, Criterion};
use seqdiff;

fn diff_long(c: &mut Criterion) {
    let s = black_box(vec![1, 2, 3, 3, 1, 2, 3, 3].repeat(300));
    let t = black_box(vec![1, 2, 2, 1, 2, 3, 3].repeat(300));
    c.bench_function("diff long", |b| b.iter(|| seqdiff::diff(&s, &t)));
}

fn diff_short(c: &mut Criterion) {
    let s = black_box(vec![1, 2, 3, 3, 1, 2, 3, 3].repeat(10));
    let t = black_box(vec![1, 2, 2, 1, 2, 3, 3].repeat(10));
    c.bench_function("diff short", |b| b.iter(|| seqdiff::diff(&s, &t)));
}

fn diff_fool(c: &mut Criterion) {
    let s = black_box(vec![0; 8].repeat(100));
    let t = black_box(vec![1; 7].repeat(100));
    c.bench_function("diff fool", |b| b.iter(|| seqdiff::diff(&s, &t)));
}

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

criterion_group!(benches, bench_ratios, diff_long, diff_short, diff_fool);
criterion_main!(benches);
