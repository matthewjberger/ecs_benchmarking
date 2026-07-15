# ecs_benchmarking

A focused, head-to-head benchmark suite that pits [freecs](https://crates.io/crates/freecs)
against [bevy_ecs](https://crates.io/crates/bevy_ecs) on the same workloads, using
[Criterion](https://crates.io/crates/criterion) for measurement.

The scenarios are lifted from the well-known
[ecs_bench_suite](https://github.com/rust-gamedev/ecs_bench_suite) so the numbers are
comparable to the wider ecosystem, but this repo keeps only the two implementations that
matter here and adds a reporting layer on top: one command runs everything, prints a
comparison table in the terminal, and writes a Markdown report.

## What it measures

Each scenario is implemented twice, once against `freecs` and once against `bevy_ecs`,
with identical component layouts and identical work per entity. Both implementations
expose the same `Benchmark::new()` / `run()` shape, and Criterion times the `run()` call.

| Scenario | What it exercises |
| --- | --- |
| `simple_insert` | Spawning 10,000 entities that each carry four components. Measures raw archetype allocation and bulk insertion. |
| `simple_iter` | Iterating 10,000 entities and folding velocity into position. Measures dense, cache-friendly read/write iteration. |
| `fragmented_iter` | Iterating one shared component spread across 26 distinct archetypes. Measures how well each library tolerates archetype fragmentation. |
| `heavy_compute` | Inverting a matrix 100 times per entity over 1,000 entities, in parallel. Measures parallel iteration and per-entity compute throughput. |
| `add_remove` | Adding then removing a component on 10,000 entities. Measures the cost of structural change and archetype migration. |
| `schedule` | Running three systems over four overlapping archetypes (40,000 entities). Measures system dispatch and query setup overhead. |

The two libraries are deliberately used the way each is meant to be used, so the
comparison reflects idiomatic code rather than a lowest-common-denominator port. `freecs`
uses its `ecs!` macro, closure-based queries, and `par_for_each_mut`; `bevy_ecs` uses
`spawn_batch`, `World::query`, `par_iter_mut`, and a `Schedule`.

## Running it

Everything runs through the [`just`](https://github.com/casey/just) recipes. The one you
want almost always is:

```
just run
```

That builds in release, runs every scenario through Criterion (which prints its own
per-benchmark timings and confidence intervals as it goes), then prints a side-by-side
comparison table and writes a Markdown report. Run `just` on its own to see the rest of
the recipes, including running a single scenario or regenerating the report without
re-benchmarking.

The comparison table looks like this:

```
  freecs vs bevy_ecs
  Scenario                                      freecs        bevy          winner    speedup
  ----------------------------------------------------------------------------------------------
  Insert 10K entities, 4 components each        197.94 us     241.54 us     freecs    1.22x
  Iterate 10K entities, position += velocity    7.89 us       6.61 us       bevy      1.19x
  Iterate Data across 26 fragmented archetypes  207.5 ns      1.03 us       freecs    4.96x
  ...
```

The winning column is highlighted in green in a real terminal. The numbers above are
illustrative; run it on your own hardware to get numbers that mean anything.

## Reports

The report layer reads Criterion's own JSON output (under `target/criterion/`) rather than
re-timing anything, so `just report` is instant and can be re-run at any time after a
benchmark pass.

Reports are written to `./reports`, which is generated and git-ignored:

- `reports/latest.md` is overwritten on every run and is the one to look at.
- `reports/report-<unix-seconds>.md` is a timestamped copy, so a sequence of runs leaves a
  trail you can diff.

Each report records the mean time per scenario for both libraries, which one won, and the
ratio between them. Criterion's full HTML reports, with plots and distributions, live
under `target/criterion/report/index.html` if you want to drill in.

## Methodology and caveats

- Times are Criterion's mean estimate of a single `run()` call. Lower is better; the
  speedup column is simply the slower time divided by the faster one.
- Microbenchmarks measure exactly what they measure and nothing more. A win here does not
  translate linearly to a win in a real game loop, where memory layout, system count, and
  contention all shift the picture. Treat these as directional signals, not a scoreboard.
- Parallel scenarios depend on core count and thread-pool warmup, so they vary the most
  between machines.
- Build with a warm cache and a quiet machine. The release profile in `Cargo.toml` uses
  thin LTO and a single codegen unit so both libraries are measured with optimizations that
  a shipping build would actually use.
- The compared versions are pinned in `Cargo.toml`. Bump them there to benchmark against a
  newer `freecs` or `bevy_ecs`.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
