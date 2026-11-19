use cgmath::*;
use hemlock_ecs::storage::{Storage, spawn_batch};

#[derive(Clone, Copy)]
struct Transform(Matrix4<f32>);

#[derive(Clone, Copy)]
struct Position(Vector3<f32>);

#[derive(Clone, Copy)]
struct Rotation(Vector3<f32>);

#[derive(Clone, Copy)]
struct Velocity(Vector3<f32>);

fn fill(storage: &mut Storage) {
    spawn_batch(
        storage,
        (0..10_000).map(|_| {
            (
                Transform(Matrix4::from_scale(1.0)),
                Position(Vector3::unit_x()),
                Rotation(Vector3::unit_x()),
                Velocity(Vector3::unit_x()),
            )
        }),
    );
}

pub struct Benchmark;

impl Benchmark {
    pub fn setup() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut storage = Storage::default();
        fill(&mut storage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemlock_ecs::storage::{living, query};

    #[test]
    fn spawn_creates_all_entities() {
        let mut storage = Storage::default();
        fill(&mut storage);
        assert_eq!(living(&storage).len(), 10_000);
        assert_eq!(query::<&Position>(&storage).count(), 10_000);
    }
}
