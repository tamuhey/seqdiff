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

criterion_group!(benches, diff_long, diff_short, diff_fool);
criterion_main!(benches);
