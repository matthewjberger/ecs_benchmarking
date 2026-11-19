use hemlock_ecs::entity::Entity;
use hemlock_ecs::storage::{Storage, forget_removals, remove, set, spawn_with};
use hemlock_ecs::system::System;

#[derive(Clone, Copy)]
struct A(f32);

#[derive(Clone, Copy)]
struct B(f32);

pub struct Benchmark(Storage, Vec<Entity>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut storage = Storage::default();
        let entities = (0..10_000)
            .map(|_| spawn_with(&mut storage, (A(0.0),)))
            .collect();

        Self(storage, entities)
    }

    pub fn run(&mut self) {
        for &entity in &self.1 {
            set(&mut self.0, entity, B(0.0));
        }

        for &entity in &self.1 {
            remove::<B>(&mut self.0, entity);
        }

        forget_removals(&mut self.0, std::iter::empty::<&System>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemlock_ecs::storage::{living, query, removed_since};

    #[test]
    fn adds_then_removes_on_every_entity() {
        let mut bench = Benchmark::setup();
        assert_eq!(living(&bench.0).len(), 10_000);

        for &entity in &bench.1 {
            set(&mut bench.0, entity, B(0.0));
        }
        assert_eq!(query::<&B>(&bench.0).count(), 10_000);

        for &entity in &bench.1 {
            remove::<B>(&mut bench.0, entity);
        }
        assert_eq!(query::<&B>(&bench.0).count(), 0);
    }

    #[test]
    fn run_leaves_no_removals_in_the_log() {
        let mut bench = Benchmark::setup();

        bench.run();

        assert_eq!(query::<&B>(&bench.0).count(), 0);
        assert_eq!(removed_since::<B>(&bench.0, 0).len(), 0);
    }
}
