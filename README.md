# ecs_benchmarking

A focused benchmark suite that runs the same workloads across several Rust Entity Component
System libraries and reports them side by side, using
[Criterion](https://crates.io/crates/criterion) for measurement.

It currently compares:

- [freecs](https://crates.io/crates/freecs)
- [bevy_ecs](https://crates.io/crates/bevy_ecs)
- [sky_ecs](https://crates.io/crates/sky_ecs)
- hemlock-ecs, pinned to a git commit of a private repository

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
| `wide_iter` | Iterating an eight-component query over 8,000,000 entities. The working set is 256 MB, which is larger than any current CPU cache, so this measures storage layout under memory pressure rather than per-operation overhead. |
| `changed_iter` | hemlock-ecs only. One system moves 1,000 of 10,000 entities, then a second system collects the entities whose position changed. Measures the row loop with change ticks on, and the read of the changes. |
| `deferred_edits` | hemlock-ecs only. Four systems queue 4,000 deferred edits (1,000 each of spawn, set, remove and despawn), and the schedule applies them. Measures the edit queue and the structural work behind it. |

Each library is used the way it is meant to be used, so the comparison reflects idiomatic
code rather than a lowest-common-denominator port. `freecs` uses its dynamic world
(`DynWorld`), typed reference queries, `spawn_bundles`, and `par_for_each`; `bevy_ecs` uses
`spawn_batch`, `World::query`, `par_iter_mut`, and a `Schedule`; `sky_ecs` uses its typed
`query_mut`, `ParView` systems, and staged `tick`;
`hemlock-ecs` uses its free functions on `Storage`, `spawn_batch`, `storage::each_mut` for
the three iteration scenarios, systems that take a `Mut` parameter with `each_mut` or
`each_mut_split` for `heavy_compute` and `schedule`, and batches from `order::batch`.
Because the APIs differ, this measures idiomatic usage of each library, not identical
instruction sequences. Read the
[Methodology](#methodology-fairness-and-what-the-numbers-do-and-dont-say) section before
drawing conclusions: two of the seven shared scenarios are deliberately not apples-to-apples,
and two more scenarios exist for hemlock-ecs only.

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
  Scenario                                      freecs      bevy        skyecs      fastest     speedup
  -------------------------------------------------------------------------------------------------------
  Insert 10K entities, 4 components each        47.56 us    248.85 us   76.92 us    freecs      1.62x
  Iterate 10K entities, position += velocity    4.87 us     7.45 us     5.68 us     freecs      1.17x
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
2. Create `src/<lib>.rs` plus `src/<lib>/<scenario>.rs` for each of the seven shared scenarios,
   each exposing a `Benchmark` with `setup()` and `run()`. Copy an existing library's
   folder as a starting point; the component layouts and per-entity work must match.
3. Add `pub mod <lib>;` to `src/lib.rs`.
4. Import the module and add one `run_impl!(group, $module, <lib>, "<label>");` line inside
   the `scenario!` macro in `benches/benchmarks.rs`.

The report picks the new column up on the next run and slots it in after the known
libraries, so nothing else needs editing.

## Methodology, fairness, and what the numbers do (and don't) say

Read this before quoting any result. Microbenchmarks are easy to misread, and two of the
seven shared scenarios compare things that are not the same underneath.

### What is and isn't timed

Criterion times only `run()`. Each library's `setup()` (building the world, spawning
fixtures, registering systems) runs once and is excluded, with one deliberate exception:

- **`simple_insert`** builds a fresh `World` and fills it inside `run()`, because creating
  and populating a world is the thing being measured. So this scenario includes the cost of
  `World::new()` on every iteration. That is consistent across all four libraries and
  matches the upstream suite, but it means `simple_insert` is "world creation plus 10K
  inserts," not inserts in isolation.
- Every other scenario builds the world once in `setup()` and times only the operation
  under test.

### The same work through different APIs

Each scenario does the same logical work with the same component layout, but through each
library's idiomatic API. Where an operation has several forms (for example a deferred
command buffer versus direct mutation), all four use the same form so they line up:
`add_remove` uses direct structural changes on every library, not command buffers. The
result is a comparison of idiomatic usage, not of identical machine code.

### freecs is measured through its dynamic world

The `freecs` column is `freecs::dynamic::DynWorld`: components registered at runtime,
queried as typed references (`query::<(&mut Position, &Velocity)>()`). freecs also ships a
static `ecs!` macro tier over the same storage, which this suite does not measure.

From freecs 5.0 a type must be marked once with `impl_component!` before it can spawn as
part of a bundle. That is a compile-time marker whose supertraits are the component storage
bounds it already met, so it costs nothing at runtime, but it is a visible difference from
sky_ecs and hemlock-ecs, where a component is never declared.

Two freecs settings are off by default and stay off here, because sky_ecs and bevy_ecs are
not paying for them either: per-row change detection (`set_change_detection`) and the
structural-change log (`structural_logging`). A world that turns them on pays two extra
`u32` columns per component, a tick stamp per row on every mutable query, and a log entry
per entity per structural change. Turn them on and these numbers get worse; that is the
feature working, not the storage.

### freecs runs its contiguous storage backend

freecs ships a `raw_storage` feature that replaces the default per-column `Box<dyn Any>`
holding a `Vec<T>` with a contiguous byte buffer reached through pointer casts, which
removes one downcast per column per table. The public API and the observable behavior are
identical across the two backends: the backend decides how a data column is held and
nothing else, and change detection and the structural log are orthogonal to it and stay
off here.

The `freecs` column runs with `raw_storage` on. bevy_ecs and sky_ecs have no equivalent
feature to turn on, which is a real difference between the libraries rather than a setting
this suite chose. hemlock-ecs has a `raw-columns` feature that is in its default features,
and this suite leaves it on: a column is then a pointer, a length and a capacity. With
`default-features = false` hemlock-ecs holds each column as a `Box<dyn Any>` over a
`Vec<T>` and has no `unsafe` code, which is the layout that `raw_storage` replaces in
freecs. Read `simple_insert`, `fragmented_iter`, and `add_remove` as "each library's fastest
configuration that this suite turns on," which for freecs and hemlock-ecs means one
contiguous column per component.

### hemlock-ecs is pinned to a git commit of a private repository

`hemlock-ecs` is the ECS crate of the hemlock engine. The `hemlock` column is a git
dependency pinned to one commit in `Cargo.toml`. The repository is private, so only its
owner can build this suite. Bump the `rev` there to benchmark against a newer commit.

hemlock-ecs was tuned against this suite, on the machine that produced the committed
report. Its column placement and its block size (both below) were picked from
measurements of `simple_iter`, `schedule` and `wide_iter` on a Ryzen 7 7800X3D. Read a
small gap in favor of the `hemlock` column with that in mind.

Seven properties of hemlock-ecs at this commit affect how its column reads:

- **Row change ticks start on the first read.** Every column keeps one tick for its last
  change, which every write through a `Mut` parameter updates. A column holds a `u64` tick
  per row only after some code reads the row changes of that component, for example with
  `changed_since`. None of the seven shared scenarios reads row changes, so there the
  `hemlock` column runs without row ticks, as the `freecs` column does with its change detection off. A game that reads
  the changes of a component pays one more `u64` write per row for that component.
  The hemlock-only `changed_iter` scenario measures that path. `changed_since` reads the
  row ticks in groups of four, with one test for each group that has no branch. A scan with
  one branch for each row measured 7.0 us or 9.4 us for the same source, and the level
  depended on where the linker put the loop. The scan by groups measured 7.0 us in both
  builds.
- **Mutable iteration has two forms.** `storage::each_mut` takes the storage exclusively
  and runs with no system, like the direct queries of freecs and sky_ecs. `simple_iter`,
  `fragmented_iter` and `wide_iter` use it. With `raw-columns` on it keeps the column
  pointers of each matching archetype between calls, and it reads them again only after a
  spawn, a despawn or a row move. The second form is a system with a `Mut` parameter, which
  takes the written columns out of the storage, runs, and puts them back. `heavy_compute`
  and `schedule` use it. A system keeps the pointers of its written columns in the same way,
  but only for columns with no row ticks, and it finds its read columns again in each run.
- **Deferred edits go into typed lanes.** `later::spawn`, `later::set` and the other edit
  calls store the edit by value in a queue for each edit type, with no allocation for each
  edit. The schedule applies a run of edits of one type in one call, and a run of spawns
  of one bundle fills each column one time. The hemlock-only `deferred_edits` scenario
  measures that path. It measured 198 us with one boxed closure for each edit, 110 us with
  the lanes, and 88 us with the spawn runs and with no removal log for unread components.
- **Columns are raw behind a default feature.** With `raw-columns` on, all `unsafe` code of
  hemlock-ecs is in one file, and each function there checks the component type and the
  bounds before it touches memory. A row move between archetypes still calls one function
  pointer per component.
- **Large columns have a fixed place in a memory page.** A column of 32 KB or more starts at
  a cache line that its component number picks, so two columns of one loop do not share
  cache sets. This is why `simple_iter`, `schedule` and `wide_iter` do not depend on where
  the allocator puts the columns.
- **Large archetypes are visited in blocks.** `storage::each_mut` visits an archetype of
  262,144 rows or more in blocks of 16 KB of its widest written column, and it prefetches
  one line of each column for the block after the next one. Only `wide_iter` is that large.
  This measured 5.83 ms without the blocks and 5.08 ms with them.
- **Removals are logged only for a component that has a reader.** `storage::remove` keeps
  the tick of the last removal of a component. It appends to the per-component log only
  after `removed_since` read that component for the first time. No code reads the removals
  of `B` in `add_remove`, so the scenario writes no log. The engine calls `forget_removals`
  once per frame, so the scenario still calls it once per `run()`, inside the timed region.

### Two scenarios are not apples-to-apples

- **`schedule`** compares each library's default scheduler, and those are architecturally
  different amounts of work. freecs runs a bare serial `for` loop over three closures, with
  no access analysis and no deferred structural changes. bevy_ecs runs its executor, which
  tracks per-system data access, applies deferred command buffers between systems, and
  carries first-run initialization; with the versions pinned here it runs single-threaded
  (bevy_ecs's `multi_threaded` feature is off by default), so the cost is executor
  machinery, not thread coordination. sky_ecs runs its own staged tick, which compiles the
  three systems into conflict-free waves once and then dispatches them; its `cd` and `ce`
  systems both write `C`, so they land in different waves and it runs this row serially
  rather than across threads. hemlock-ecs
  derives access from the system parameters, so `order::batch` puts `ab` and `cd` in one
  batch and `ce` in a second one. Its `schedule::run` times each system on one run in eight
  and only sends a batch to rayon when the averaged time of the batch is above 100 microseconds,
  so a batch this small can run inline. Read this row as
  "the cost of dispatching three trivial systems the way each library does it out of the
  box," not as a controlled measurement of dispatch cost. A library that does more per run
  (safety checks, deferred edits) looks slower here even though that machinery pays off in a
  real frame with many heavy, interacting systems.
- **`heavy_compute`** is dominated by the matrix math (100 inversions per entity through
  cgmath), not by ECS storage, which is why freecs, bevy_ecs and sky_ecs land within a few
  percent of each other. It also runs three different parallel backends (freecs and
  hemlock-ecs on rayon, bevy on its task pool, sky on its own pool), each with its own
  warmup and chunking. Read it as "do they parallelize a compute-heavy loop without falling
  over," not as a storage benchmark. The `hemlock` column is the exception, and it is not
  like-for-like. `each_mut_parallel` does not split an archetype of 2,048 rows or fewer, so
  this scenario uses `each_mut_split` with a least size of 16 rows, which spreads the 1,000
  rows over the cores. The other three columns use the default chunking of their parallel
  call, and their times show that those defaults do not spread 1,000 rows well. The gap in
  this row is a difference in chunk size, not in storage.

The four scenarios that *are* clean like-for-like comparisons are `simple_insert`,
`simple_iter`, `fragmented_iter`, and `add_remove`. For the `hemlock` column, `add_remove`
carries the last-removal tick and the `forget_removals` call that the hemlock-ecs section
above describes.

### Two scenarios are hemlock-ecs only

`changed_iter` and `deferred_edits` have a `hemlock` column and no other. They measure two
paths that games built on hemlock-ecs use in each frame and that the seven shared
scenarios do not reach: the row loop with change ticks on, and the deferred edit queue.
The other libraries have no module for them here, so their cells show `-` and the report
names no winner for these rows. Do not read them as a comparison.

### Statistics

- The table ranks on Criterion's **mean**. For most rows the mean, median, and confidence
  interval agree to within about a percent. Some rows can be outlier-skewed on a busy
  machine (the mean sits above the median when a few samples run slow); when that happens
  the median is the more honest number. If you want the ranking robust to stray slow
  samples, switch `read_mean` in `src/bin/report.rs` to read `median.point_estimate`.
- Numbers come from one machine, one run, warm cache. Run it yourself, and treat any gap
  under roughly 10 percent as noise rather than a ranking.

### Scale

Six of the seven shared scenarios use 1K to 40K entities, which fit comfortably in cache. They
measure per-operation overhead and iteration tightness, not memory-bandwidth behavior. A
library that wins at 10K can lose at 10M, and the reverse.

`wide_iter` is the counterweight. It uses 8,000,000 entities with eight `f32` components
each, so one pass touches 256 MB of component data. That is larger than any current CPU
cache, which makes it a test of storage layout rather than of per-call overhead. Note that
a large cache moves this threshold a long way: on a Ryzen 7 7800X3D the 96 MB of L3 keeps a
32 MB working set fully resident, so a smaller version of this scenario measures nothing
useful. Size it against the machine you run it on.

`wide_iter` is listed last on purpose. Its working set evicts everything, so running it
ahead of the small scenarios would change their times.

### `wide_iter` does not read the entity vector

`freecs` and `hemlock-ecs` hand the entity to the closure; `sky_ecs` and `bevy_ecs` do
not. That asymmetry would be an extra 64 MB stream for two of the four columns, which at
this scale would be large enough to explain a result on its own.

For freecs it is expected not to apply, because the closure ignores the entity and the
optimizer can remove the load. That was measured on a `nightshade-ecs` column that this
suite no longer has, which used the same closure form as freecs: forcing the entity to
stay live, by folding `entity.id` into a checksum passed to `black_box`, moved that column
from 6.27 ms to 7.94 ms. The 1.67 ms difference is the cost of that 64 MB stream, and it
was absent from the reported figure.

For the `hemlock` column the generated assembly was read at hemlock commit `1a43ca20`: the
loop is vectorized four rows wide, it loads the eight component columns and nothing else,
and it stores no row change tick, because no code of this scenario reads row changes. The
later block loop of `storage::each_mut` runs the same row code on 4,096 rows at a time.

If you change this scenario, re-check that property before comparing the columns.

### Results are not checksummed

Following the upstream suite, `run()` performs the work but does not verify the output. In
practice the mutations go through non-inlined library calls into heap storage, which stops
the optimizer from deleting them, so the numbers reflect real work. If you extend the suite
and see an implausibly fast result, wrap an aggregate of the output in `black_box` to rule
out dead-code elimination before trusting it.

### Component payloads

Component fields are written on spawn and relocated by the ECS but never read back through a
field accessor, so the compiler reports them as never-read. That is expected: they are
realistic payload the storage has to carry and move between archetypes. Every implementation
module carries a scoped `#![allow(dead_code)]` for exactly this reason.

### Build settings and versions

The release profile in `Cargo.toml` uses thin LTO and a single codegen unit, so every
library is measured with the kind of optimization a shipping build would actually use. The
compared versions, and the feature set each library runs with, are pinned in `Cargo.toml`;
bump them there to benchmark against newer releases.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
