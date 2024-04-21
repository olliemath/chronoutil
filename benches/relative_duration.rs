use criterion::{black_box, criterion_group, criterion_main, Criterion};

use chronoutil::RelativeDuration;

fn relative_duration_format_benchmark(c: &mut Criterion) {
    let durations = [
        "P1M",
        "P1Y1M1W1DT1H1M1S",
        "P99999999Y11M30DT23H59M59.999999999S",
    ]
    .iter()
    .map(|s| RelativeDuration::from_iso_8601(s).unwrap())
    .collect::<Vec<RelativeDuration>>();

    let mut g = c.benchmark_group("relative_duration_format");

    g.bench_function("one_specifier", |b| {
        b.iter(|| black_box(durations[0]).to_iso_8601())
    });
    g.bench_function("all_specifiers", |b| {
        b.iter(|| black_box(durations[1]).to_iso_8601())
    });
    g.bench_function("long_specifiers", |b| {
        b.iter(|| black_box(durations[2]).to_iso_8601())
    });
}

fn relative_duration_parse_benchmark(c: &mut Criterion) {
    let durations = [
        "P1M",
        "P1Y1M1W1DT1H1M1S",
        "P99999999Y11M30DT23H59M59.999999999S",
    ];

    let mut g = c.benchmark_group("relative_duration_parse");

    g.bench_function("one_specifier", |b| {
        b.iter(|| RelativeDuration::from_iso_8601(black_box(durations[0])))
    });
    g.bench_function("all_specifiers", |b| {
        b.iter(|| RelativeDuration::from_iso_8601(black_box(durations[1])))
    });
    g.bench_function("long_specifiers", |b| {
        b.iter(|| RelativeDuration::from_iso_8601(black_box(durations[2])))
    });
}

criterion_group!(
    benches,
    relative_duration_format_benchmark,
    relative_duration_parse_benchmark
);
criterion_main!(benches);
