use cgmath::*;
use hemlock_ecs::storage::{Storage, each_mut, spawn_with};

#[derive(Clone, Copy)]
struct Transform(Matrix4<f32>);

#[derive(Clone, Copy)]
struct Position(Vector3<f32>);

#[derive(Clone, Copy)]
struct Rotation(Vector3<f32>);

#[derive(Clone, Copy)]
struct Velocity(Vector3<f32>);

pub struct Benchmark(Storage);

impl Benchmark {
    pub fn setup() -> Self {
        let mut storage = Storage::default();
        for _ in 0..10_000 {
            spawn_with(
                &mut storage,
                (
                    Transform(Matrix4::from_scale(1.0)),
                    Position(Vector3::unit_x()),
                    Rotation(Vector3::unit_x()),
                    Velocity(Vector3::unit_x()),
                ),
            );
        }

        Self(storage)
    }

    pub fn run(&mut self) {
        each_mut::<(Position,), (&Velocity,)>(
            &mut self.0,
            |_entity, (mut position,), (velocity,)| {
                position.0 += velocity.0;
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemlock_ecs::storage::{living, query};

    #[test]
    fn iterates_all_entities_and_mutates() {
        let mut bench = Benchmark::setup();
        assert_eq!(living(&bench.0).len(), 10_000);

        bench.run();

        let mut count = 0;
        let mut sample_x = 0.0;
        for (_entity, position) in query::<&Position>(&bench.0) {
            count += 1;
            sample_x = position.0.x;
        }
        assert_eq!(count, 10_000);
        assert!((sample_x - 2.0).abs() < 1e-3);
    }
}
