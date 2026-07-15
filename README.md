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
| `heavy_compute` | Inverting a matrix 100 times per entity over 1,000 entities, in parallel. Compute-bound, not a storage test (see Methodology). |
| `add_remove` | Adding then removing a component on 10,000 entities. Measures the cost of structural change and archetype migration. |
| `schedule` | Running three systems over four overlapping archetypes (40,000 entities). Compares each library's own scheduler, which are not the same machine (see Methodology). |

Each library is used the way it is meant to be used, so the comparison reflects idiomatic
code rather than a lowest-common-denominator port. `freecs` uses its `ecs!` macro,
closure-based queries, and `par_for_each_mut`; `bevy_ecs` uses `spawn_batch`,
`World::query`, `par_iter_mut`, and a `Schedule`; `sky_ecs` uses its typed `query_mut`,
`ParView` systems, and staged `tick`. Because the APIs differ, this measures idiomatic
usage of each library, not identical instruction sequences. Read the
[Methodology](#methodology-fairness-and-what-the-numbers-do-and-dont-say) section before
drawing conclusions: two of the six scenarios are deliberately not apples-to-apples.

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

## Methodology, fairness, and what the numbers do (and don't) say

Read this before quoting any result. Microbenchmarks are easy to misread, and two of the
six scenarios compare things that are not the same underneath.

### What is and isn't timed

Criterion times only `run()`. Each library's `setup()` (building the world, spawning
fixtures, registering systems) runs once and is excluded, with one deliberate exception:

- **`simple_insert`** builds a fresh `World` and fills it inside `run()`, because creating
  and populating a world is the thing being measured. So this scenario includes the cost of
  `World::new()` on every iteration. That is consistent across all three libraries and
  matches the upstream suite, but it means `simple_insert` is "world creation plus 10K
  inserts," not inserts in isolation.
- Every other scenario builds the world once in `setup()` and times only the operation
  under test.

### The same work through different APIs

Each scenario does the same logical work with the same component layout, but through each
library's idiomatic API. Where an operation has several forms (for example a deferred
command buffer versus direct mutation), all three use the same form so they line up:
`add_remove` uses direct structural changes on every library, not command buffers. The
result is a comparison of idiomatic usage, not of identical machine code.

### Iteration API overhead is real and measurable

freecs's ergonomic query hands the closure `(entity, &mut table, index)`, so you write
`table.position[index]`, a bounds-checked index on every access. sky_ecs hands the closure
direct references. On `simple_iter` that shows up as a gap, but the gap is the convenience
API, not the storage layout. freecs also exposes its columns as plain public `Vec`s, so a
hot loop can iterate the raw slices instead. The `experiment_iter` bench (`just experiment`)
measures all three over the identical component layout:

| Iteration path | Time (10K entities) |
| --- | --- |
| freecs, ergonomic `table.position[idx]` | ~8.0 us |
| freecs, raw `world.tables` column slices | ~5.0 us |
| sky_ecs, `query_mut` references | ~5.1 us |

Iterating freecs's raw column slices ties sky_ecs; the whole ~1.6x gap on `simple_iter` was
bounds-checked indexing in the convenience API, not the underlying storage. The suite's
`simple_iter` row uses the ergonomic API on every library, because that is how people
normally write each one, but the storage layouts are effectively equal for dense iteration.
The experiment checksums each pass, so the numbers are verified to do the work.

### Two scenarios are not apples-to-apples

- **`schedule`** compares each library's default scheduler, and those are architecturally
  different amounts of work. freecs runs a bare serial `for` loop over three closures, with
  no access analysis and no deferred structural changes. bevy_ecs runs its executor, which
  tracks per-system data access, applies deferred command buffers between systems, and
  carries first-run initialization; with the versions pinned here it runs single-threaded
  (bevy_ecs's `multi_threaded` feature is off by default), so the cost is executor
  machinery, not thread coordination. sky_ecs runs its own staged tick. Read this row as
  "the cost of dispatching three trivial systems the way each library does it out of the
  box," not as a controlled measurement of dispatch cost. A library that does more per run
  (safety checks, deferred edits) looks slower here even though that machinery pays off in a
  real frame with many heavy, interacting systems.
- **`heavy_compute`** is dominated by the matrix math (100 inversions per entity through
  cgmath), not by ECS storage, which is why the three land within a few percent of each
  other. It also runs three different parallel backends (freecs on rayon, bevy on its task
  pool, sky on its own pool), each with its own warmup and chunking. Read it as "do all
  three parallelize a compute-heavy loop without falling over," not as a storage benchmark.

The four scenarios that *are* clean like-for-like comparisons are `simple_insert`,
`simple_iter`, `fragmented_iter`, and `add_remove`.

### Statistics

- The table ranks on Criterion's **mean**. For most rows the mean, median, and confidence
  interval agree to within about a percent. Some rows can be outlier-skewed on a busy
  machine (the mean sits above the median when a few samples run slow); when that happens
  the median is the more honest number. If you want the ranking robust to stray slow
  samples, switch `read_mean` in `src/bin/report.rs` to read `median.point_estimate`.
- Numbers come from one machine, one run, warm cache. Run it yourself, and treat any gap
  under roughly 10 percent as noise rather than a ranking.

### Scale

Entity counts here (1K to 40K) fit comfortably in cache, so these measure per-operation
overhead and iteration tightness, not memory-bandwidth behavior at millions of entities. A
library that wins at 10K can lose at 10M, and the reverse. If you care about a specific
scale, change the counts in the scenario modules and re-run.

### Results are not checksummed

Following the upstream suite, `run()` performs the work but does not verify the output. In
practice the mutations go through non-inlined library calls into heap storage, which stops
the optimizer from deleting them, so the numbers reflect real work. If you extend the suite
and see an implausibly fast result, wrap an aggregate of the output in `black_box` to rule
out dead-code elimination before trusting it.

### Component payloads

Component fields are written on spawn and relocated by the ECS but never read back through a
field accessor, so the compiler reports them as never-read. That is expected: they are
realistic payload the storage has to carry and move between archetypes. The `bevy` and `sky`
modules carry a scoped `#![allow(dead_code)]` for exactly this reason; the freecs components
expose public fields and so do not trip the lint.

### Build settings and versions

The release profile in `Cargo.toml` uses thin LTO and a single codegen unit, so every
library is measured with the kind of optimization a shipping build would actually use. The
compared versions are pinned in `Cargo.toml`; bump them there to benchmark against newer
releases.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
