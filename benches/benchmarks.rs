use criterion::{Criterion, criterion_group, criterion_main};
use ecs_benchmarking::{bevy, freecs_dyn, hemlock, sky};

macro_rules! run_impl {
    ($group:expr, $module:ident, $lib:ident, $label:literal) => {
        $group.bench_function($label, |bencher| {
            let mut bench = $lib::$module::Benchmark::setup();
            bencher.iter(move || bench.run());
        });
    };
}

// To add another ECS library: create `src/<lib>/<scenario>.rs` modules that
// expose `Benchmark::setup()` / `run()`, import the module below, and add one
// `run_impl!` line per library here. The report auto-discovers every column.
macro_rules! scenario {
    ($criterion:expr, $group:literal, $module:ident) => {{
        let mut group = $criterion.benchmark_group($group);
        run_impl!(group, $module, freecs_dyn, "freecs");
        run_impl!(group, $module, bevy, "bevy");
        run_impl!(group, $module, sky, "skyecs");
        run_impl!(group, $module, hemlock, "hemlock");
        group.finish();
    }};
}

// A scenario that only hemlock has a module for: it measures a path that the
// other libraries have no like-for-like form of in this suite.
macro_rules! hemlock_scenario {
    ($criterion:expr, $group:literal, $module:ident) => {{
        let mut group = $criterion.benchmark_group($group);
        run_impl!(group, $module, hemlock, "hemlock");
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
    scenario!(criterion, "wide_iter", wide_iter);
    hemlock_scenario!(criterion, "changed_iter", changed_iter);
    hemlock_scenario!(criterion, "deferred_edits", deferred_edits);
}

criterion_group!(benchmarks, benches);
criterion_main!(benchmarks);
