// Focused experiment: does freecs's dynamic tier fall behind on a single large
// archetype because its parallel iteration is table-granular (one archetype
// runs on one core), while bevy and sky_ecs chunk-parallelize within an
// archetype? One archetype, one million entities, a compute-bound per-entity
// kernel. All three start from identical state, so the printed checksums must
// match, which cross-validates that they perform the same work.
//
// Run with: cargo bench --bench experiment_parallel (or `just experiment-parallel`)

#![allow(dead_code)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

const ENTITIES: usize = 1_000_000;
const ITERS: usize = 16;

#[inline]
fn kernel(px: &mut f32, py: &mut f32, pz: &mut f32, vx: f32, vy: f32, vz: f32) {
    for _ in 0..ITERS {
        *px = (*px + vx).sin();
        *py = (*py + vy).cos();
        *pz = (*pz * 1.0001 + vz).abs().sqrt();
    }
}

mod freecs_dyn_mod {
    use super::*;
    use freecs::dynamic::DynWorld;

    #[derive(Clone, Copy, Default)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Clone, Copy, Default)]
    struct Velocity {
        x: f32,
        y: f32,
        z: f32,
    }

    pub struct Bench(DynWorld);

    impl Bench {
        pub fn build() -> Self {
            let mut world = DynWorld::new();
            world.spawn_bundles(
                (
                    Position {
                        x: 0.1,
                        y: 0.5,
                        z: 1.0,
                    },
                    Velocity {
                        x: 0.001,
                        y: 0.002,
                        z: 0.003,
                    },
                ),
                ENTITIES,
            );
            Self(world)
        }

        pub fn run(&mut self) {
            self.0.query::<(&mut Position, &Velocity)>().par_for_each(
                |_entity, (position, velocity)| {
                    kernel(
                        &mut position.x,
                        &mut position.y,
                        &mut position.z,
                        velocity.x,
                        velocity.y,
                        velocity.z,
                    );
                },
            );
        }

        pub fn checksum(&mut self) -> f64 {
            let mut sum = 0.0f64;
            self.0
                .query::<&Position>()
                .for_each(|_entity, position| sum += position.x as f64);
            sum
        }
    }
}

mod bevy_mod {
    use super::*;
    use bevy_ecs::prelude::*;
    use bevy_tasks::{ComputeTaskPool, TaskPool};

    #[derive(Component, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Component, Clone, Copy)]
    struct Velocity {
        x: f32,
        y: f32,
        z: f32,
    }

    pub struct Bench(World);

    impl Bench {
        pub fn build() -> Self {
            ComputeTaskPool::get_or_init(TaskPool::default);
            let mut world = World::new();
            world.spawn_batch((0..ENTITIES).map(|_| {
                (
                    Position {
                        x: 0.1,
                        y: 0.5,
                        z: 1.0,
                    },
                    Velocity {
                        x: 0.001,
                        y: 0.002,
                        z: 0.003,
                    },
                )
            }));
            Self(world)
        }

        pub fn run(&mut self) {
            let mut query = self.0.query::<(&mut Position, &Velocity)>();
            query
                .par_iter_mut(&mut self.0)
                .for_each(|(mut position, velocity)| {
                    let position = &mut *position;
                    kernel(
                        &mut position.x,
                        &mut position.y,
                        &mut position.z,
                        velocity.x,
                        velocity.y,
                        velocity.z,
                    );
                });
        }

        pub fn checksum(&mut self) -> f64 {
            let mut query = self.0.query::<&Position>();
            let mut sum = 0.0f64;
            for position in query.iter(&self.0) {
                sum += position.x as f64;
            }
            sum
        }
    }
}

mod sky_mod {
    use super::*;
    use sky_ecs::{ParView, Update, World};

    #[derive(Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Clone, Copy)]
    struct Velocity {
        x: f32,
        y: f32,
        z: f32,
    }

    fn compute(bodies: ParView<(&mut Position, &Velocity)>) {
        bodies.par_for_each(|(position, velocity)| {
            kernel(
                &mut position.x,
                &mut position.y,
                &mut position.z,
                velocity.x,
                velocity.y,
                velocity.z,
            );
        });
    }

    pub struct Bench(World);

    impl Bench {
        pub fn build() -> Self {
            let mut world = World::new();
            world.spawn_batch((0..ENTITIES).map(|_| {
                (
                    Position {
                        x: 0.1,
                        y: 0.5,
                        z: 1.0,
                    },
                    Velocity {
                        x: 0.001,
                        y: 0.002,
                        z: 0.003,
                    },
                )
            }));
            world.stage(Update).add(compute);
            Self(world)
        }

        pub fn run(&mut self) {
            self.0.tick_with_delta(0.0).unwrap();
        }

        pub fn checksum(&self) -> f64 {
            let mut sum = 0.0f64;
            self.0
                .query::<&Position>()
                .for_each(|position| sum += position.x as f64);
            sum
        }
    }
}

fn experiment(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("parallel_one_archetype");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(10);

    group.bench_function("freecs_dyn", |bencher| {
        let mut bench = freecs_dyn_mod::Bench::build();
        bench.run();
        eprintln!("  verify freecs_dyn checksum: {:.4}", bench.checksum());
        bencher.iter(|| {
            bench.run();
            black_box(&mut bench);
        });
    });

    group.bench_function("bevy", |bencher| {
        let mut bench = bevy_mod::Bench::build();
        bench.run();
        eprintln!("  verify bevy checksum:       {:.4}", bench.checksum());
        bencher.iter(|| {
            bench.run();
            black_box(&mut bench);
        });
    });

    group.bench_function("skyecs", |bencher| {
        let mut bench = sky_mod::Bench::build();
        bench.run();
        eprintln!("  verify skyecs checksum:     {:.4}", bench.checksum());
        bencher.iter(|| {
            bench.run();
            black_box(&mut bench);
        });
    });

    group.finish();
}

criterion_group!(experiment_group, experiment);
criterion_main!(experiment_group);
