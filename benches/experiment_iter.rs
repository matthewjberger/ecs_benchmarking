// Focused experiment: how much of freecs's simple_iter gap versus sky_ecs is
// the bounds-checked `table.field[idx]` convenience API, and how much is the
// storage layout itself? Compares three paths over an identical component
// layout (the same one simple_iter uses):
//
//   freecs_index  freecs's ergonomic query closure, `table.position[idx]`
//   freecs_slice  hand-iterated raw column slices off the public `world.tables`
//   skyecs        sky_ecs's typed `query_mut` reference iteration
//
// Run with: cargo bench --bench experiment_iter (or `just experiment`)

// Some component fields are payload carried by storage but never read back
// through a field accessor, which the compiler reports as never-read.
#![allow(dead_code)]

use cgmath::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

mod freecs_world {
    use super::*;
    use freecs::ecs;

    ecs! {
        World {
            transform: Transform => TRANSFORM,
            position: Position => POSITION,
            rotation: Rotation => ROTATION,
            velocity: Velocity => VELOCITY,
        }
        Resources {}
    }

    #[derive(Clone, Copy)]
    pub struct Transform(pub Matrix4<f32>);
    impl Default for Transform {
        fn default() -> Self {
            Self(Matrix4::identity())
        }
    }

    #[derive(Clone, Copy)]
    pub struct Position(pub Vector3<f32>);
    impl Default for Position {
        fn default() -> Self {
            Self(Vector3::new(0.0, 0.0, 0.0))
        }
    }

    #[derive(Clone, Copy)]
    pub struct Rotation(pub Vector3<f32>);
    impl Default for Rotation {
        fn default() -> Self {
            Self(Vector3::new(0.0, 0.0, 0.0))
        }
    }

    #[derive(Clone, Copy)]
    pub struct Velocity(pub Vector3<f32>);
    impl Default for Velocity {
        fn default() -> Self {
            Self(Vector3::new(0.0, 0.0, 0.0))
        }
    }

    pub fn build() -> World {
        let mut world = World::default();
        world.spawn_batch(
            TRANSFORM | POSITION | ROTATION | VELOCITY,
            10_000,
            |table, idx| {
                table.transform[idx] = Transform(Matrix4::from_scale(1.0));
                table.position[idx] = Position(Vector3::unit_x());
                table.rotation[idx] = Rotation(Vector3::unit_x());
                table.velocity[idx] = Velocity(Vector3::unit_x());
            },
        );
        world
    }

    pub fn iter_index(world: &mut World) {
        world
            .query_mut()
            .with(POSITION | VELOCITY)
            .iter(|_entity, table, idx| {
                table.position[idx].0 += table.velocity[idx].0;
            });
    }

    pub fn iter_slice(world: &mut World) {
        for table in &mut world.tables {
            if table.mask & (POSITION | VELOCITY) == (POSITION | VELOCITY) {
                for (position, velocity) in table.position.iter_mut().zip(table.velocity.iter()) {
                    position.0 += velocity.0;
                }
            }
        }
    }

    pub fn checksum(world: &World) -> f32 {
        let mut sum = 0.0;
        world.query().with(POSITION).iter(|_entity, table, idx| {
            sum += table.position[idx].0.x;
        });
        sum
    }
}

mod sky_world {
    use super::*;
    use sky_ecs::World;

    #[derive(Clone, Copy)]
    struct Transform(Matrix4<f32>);
    #[derive(Clone, Copy)]
    struct Position(Vector3<f32>);
    #[derive(Clone, Copy)]
    struct Rotation(Vector3<f32>);
    #[derive(Clone, Copy)]
    struct Velocity(Vector3<f32>);

    pub struct Bench(World);

    impl Bench {
        pub fn build() -> Self {
            let mut world = World::new();
            world.spawn_batch((0..10_000).map(|_| {
                (
                    Transform(Matrix4::from_scale(1.0)),
                    Position(Vector3::unit_x()),
                    Rotation(Vector3::unit_x()),
                    Velocity(Vector3::unit_x()),
                )
            }));
            Self(world)
        }

        pub fn iter(&mut self) {
            self.0
                .query_mut::<(&Velocity, &mut Position)>()
                .for_each(|(velocity, position)| {
                    position.0 += velocity.0;
                });
        }

        pub fn checksum(&self) -> f32 {
            let mut sum = 0.0;
            self.0
                .query::<&Position>()
                .for_each(|position| sum += position.0.x);
            sum
        }
    }
}

fn experiment(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("iter_experiment");

    group.bench_function("freecs_index", |bencher| {
        let mut world = freecs_world::build();
        let before = freecs_world::checksum(&world);
        freecs_world::iter_index(&mut world);
        eprintln!(
            "  verify freecs_index: checksum {before} -> {}",
            freecs_world::checksum(&world)
        );
        bencher.iter(|| {
            freecs_world::iter_index(&mut world);
            black_box(&mut world);
        });
    });

    group.bench_function("freecs_slice", |bencher| {
        let mut world = freecs_world::build();
        let before = freecs_world::checksum(&world);
        freecs_world::iter_slice(&mut world);
        eprintln!(
            "  verify freecs_slice: checksum {before} -> {}",
            freecs_world::checksum(&world)
        );
        bencher.iter(|| {
            freecs_world::iter_slice(&mut world);
            black_box(&mut world);
        });
    });

    group.bench_function("skyecs", |bencher| {
        let mut bench = sky_world::Bench::build();
        let before = bench.checksum();
        bench.iter();
        eprintln!(
            "  verify skyecs:       checksum {before} -> {}",
            bench.checksum()
        );
        bencher.iter(|| {
            bench.iter();
            black_box(&bench);
        });
    });

    group.finish();
}

criterion_group!(experiment_group, experiment);
criterion_main!(experiment_group);
