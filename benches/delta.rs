use chrono::naive::NaiveDate;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use chronoutil::delta::{is_leap_year, shift_months};

fn shift_months_benchmark(c: &mut Criterion) {
    let shifts = black_box(-18..19);
    let base = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();

    c.bench_function("shift_months", |b| {
        b.iter::<Vec<NaiveDate>, _>(|| shifts.clone().map(|s| shift_months(base, s)).collect())
    });
}

fn in_month(y: i32, m: u32, d: u32) -> bool {
    if m == 4 || m == 6 || m == 9 || m == 11 {
        d <= 30
    } else if m == 2 {
        d <= 28 || (d == 29 && is_leap_year(y))
    } else {
        true
    }
}

fn shift_months_many_benchmark(c: &mut Criterion) {
    let shift = black_box(6);
    let mut bases = Vec::new();
    for y in 0..101 {
        for m in 1..13 {
            for d in 1..31 {
                if in_month(y, m, d) {
                    bases.push(NaiveDate::from_ymd_opt(y, m, d).unwrap());
                }
            }
        }
    }

    c.bench_function("shift_months_many", |b| {
        b.iter::<Vec<NaiveDate>, _>(|| bases.iter().map(|b| shift_months(*b, shift)).collect())
    });
}

criterion_group!(benches, shift_months_benchmark, shift_months_many_benchmark);
criterion_main!(benches);
