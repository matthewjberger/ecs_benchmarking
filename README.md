# ecs_benchmarking

A focused benchmark suite that runs the same workloads across several Rust Entity Component
System libraries and reports them side by side, using
[Criterion](https://crates.io/crates/criterion) for measurement.

It currently compares:

- [freecs](https://crates.io/crates/freecs)
- [bevy_ecs](https://crates.io/crates/bevy_ecs)
- [sky_ecs](https://crates.io/crates/sky_ecs)

The scenarios are lifted from the well-known
[ecs_bench_suite](https://github.com/rust-gamedev/ecs_bench_suite) so the numbers are
comparable to the wider ecosystem. On top of that, this repo adds a reporting layer: one
command runs everything, prints a comparison table in the terminal, and writes a Markdown
report. Adding another library is deliberately cheap (see below).

## What it measures

Each scenario is implemented once per library, with identical component layouts and
identical work per entity. Every implementation exposes the same `Benchmark::setup()` /
`run()` shape: `setup()` builds the world and is not timed, `run()` is what Criterion
measures.

| Scenario | What it exercises |
| --- | --- |
| `simple_insert` | Spawning 10,000 entities that each carry four components. Measures raw archetype allocation and bulk insertion. |
| `simple_iter` | Iterating 10,000 entities and folding velocity into position. Measures dense, cache-friendly read/write iteration. |
| `fragmented_iter` | Iterating one shared component spread across 26 distinct archetypes. Measures how well each library tolerates archetype fragmentation. |
| `heavy_compute` | Inverting a matrix 100 times per entity over 1,000 entities, in parallel. Measures parallel iteration and per-entity compute throughput. |
| `add_remove` | Adding then removing a component on 10,000 entities. Measures the cost of structural change and archetype migration. |
| `schedule` | Running three systems over four overlapping archetypes (40,000 entities). Measures system dispatch and query setup overhead. |

Each library is used the way it is meant to be used, so the comparison reflects idiomatic
code rather than a lowest-common-denominator port. `freecs` uses its `ecs!` macro,
closure-based queries, and `par_for_each_mut`; `bevy_ecs` uses `spawn_batch`,
`World::query`, `par_iter_mut`, and a `Schedule`; `sky_ecs` uses its typed `query_mut`,
`ParView` systems, and staged `tick`.

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
  freecs vs bevy vs skyecs
  Scenario                                      freecs      bevy        skyecs      fastest  speedup
  ----------------------------------------------------------------------------------------------------
  Insert 10K entities, 4 components each        159.94 us   248.02 us   81.92 us    skyecs   1.95x
  Iterate 10K entities, position += velocity    8.69 us     7.38 us     5.31 us     skyecs   1.39x
  ...
```

The fastest cell in each row is highlighted in green in a real terminal, and the speedup
column shows how far ahead the winner is over the next-fastest library. The numbers above
are illustrative; run it on your own hardware to get numbers that mean anything.

## Reports

The report layer reads Criterion's own JSON output (under `target/criterion/`) rather than
re-timing anything, so `just report` is instant and can be re-run at any time after a
benchmark pass. It discovers the set of libraries directly from that output, so the table
grows a column automatically when you add one.

Reports are written to `./reports`, which is generated and git-ignored:

- `reports/latest.md` is overwritten on every run and is the one to look at.
- `reports/report-<unix-seconds>.md` is a timestamped copy, so a sequence of runs leaves a
  trail you can diff.

Criterion's full HTML reports, with plots and distributions, live under
`target/criterion/report/index.html` if you want to drill in.

## Adding another ECS library

The suite is built so a new library slots in without touching the report or the existing
implementations:

1. Add the crate to `[dependencies]` in `Cargo.toml`.
2. Create `src/<lib>.rs` plus `src/<lib>/<scenario>.rs` for each of the six scenarios,
   each exposing a `Benchmark` with `setup()` and `run()`. Copy an existing library's
   folder as a starting point; the component layouts and per-entity work must match.
3. Add `pub mod <lib>;` to `src/lib.rs`.
4. Import the module and add one `run_impl!(group, $module, <lib>, "<label>");` line inside
   the `scenario!` macro in `benches/benchmarks.rs`.

The report picks the new column up on the next run and slots it in after the known
libraries, so nothing else needs editing.

## Methodology and caveats

- Times are Criterion's mean estimate of a single `run()` call. Lower is better; the
  speedup column is the next-fastest time divided by the winner's time.
- Microbenchmarks measure exactly what they measure and nothing more. A win here does not
  translate linearly to a win in a real game loop, where memory layout, system count, and
  contention all shift the picture. Treat these as directional signals, not a scoreboard.
- Parallel scenarios depend on core count and thread-pool warmup, so they vary the most
  between machines.
- Build with a warm cache and a quiet machine. The release profile in `Cargo.toml` uses
  thin LTO and a single codegen unit so every library is measured with optimizations a
  shipping build would actually use.
- The compared versions are pinned in `Cargo.toml`. Bump them there to benchmark against
  newer releases.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
