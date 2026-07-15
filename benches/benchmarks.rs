use criterion::{Criterion, criterion_group, criterion_main};
use ecs_benchmarking::{bevy, freecs_impl};

macro_rules! scenario {
    ($criterion:expr, $group:literal, $module:ident) => {{
        let mut group = $criterion.benchmark_group($group);
        group.bench_function("freecs", |bencher| {
            let mut bench = freecs_impl::$module::Benchmark::setup();
            bencher.iter(move || bench.run());
        });
        group.bench_function("bevy", |bencher| {
            let mut bench = bevy::$module::Benchmark::setup();
            bencher.iter(move || bench.run());
        });
        group.finish();
    }};
}

fn benches(criterion: &mut Criterion) {
    scenario!(criterion, "simple_insert", simple_insert);
    scenario!(criterion, "simple_iter", simple_iter);
    scenario!(criterion, "fragmented_iter", frag_iter);
    scenario!(criterion, "heavy_compute", heavy_compute);
    scenario!(criterion, "add_remove", add_remove);
    scenario!(criterion, "schedule", schedule);
}

criterion_group!(benchmarks, benches);
criterion_main!(benchmarks);
