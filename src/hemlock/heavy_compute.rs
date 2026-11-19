use cgmath::*;
use hemlock_ecs::resources::Resources;
use hemlock_ecs::schedule::once;
use hemlock_ecs::storage::{Storage, spawn_batch};
use hemlock_ecs::system::{IntoSystem, Mut, System, each_mut_split};

const LEAST_ROWS_TO_SPLIT: usize = 16;

#[derive(Clone, Copy)]
struct Position(Vector3<f32>);

#[derive(Clone, Copy)]
struct Rotation(Vector3<f32>);

#[derive(Clone, Copy)]
struct Velocity(Vector3<f32>);

#[derive(Clone, Copy)]
struct Matrix(Matrix4<f32>);

fn invert(mut rows: Mut<(Position, Matrix)>) {
    each_mut_split(
        &mut rows,
        LEAST_ROWS_TO_SPLIT,
        |_entity, (mut position, mut matrix), ()| {
            for _ in 0..100 {
                matrix.0 = matrix.0.invert().unwrap();
            }
            position.0 = matrix.0.transform_vector(position.0);
        },
    );
}

pub struct Benchmark(Storage, Resources, System);

impl Benchmark {
    pub fn setup() -> Self {
        let mut storage = Storage::default();
        spawn_batch(
            &mut storage,
            (0..1000).map(|_| {
                (
                    Matrix(Matrix4::from_angle_x(Rad(1.2))),
                    Position(Vector3::unit_x()),
                    Rotation(Vector3::unit_x()),
                    Velocity(Vector3::unit_x()),
                )
            }),
        );

        Self(storage, Resources::default(), invert.into_system())
    }

    pub fn run(&mut self) {
        once(&mut self.2, &mut self.0, &mut self.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemlock_ecs::storage::{living, query};

    #[test]
    fn processes_all_entities_in_parallel() {
        let mut bench = Benchmark::setup();
        assert_eq!(living(&bench.0).len(), 1000);

        bench.run();

        assert_eq!(query::<&Position>(&bench.0).count(), 1000);
    }
}
