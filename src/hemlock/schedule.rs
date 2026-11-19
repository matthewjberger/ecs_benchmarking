use hemlock_ecs::order::batch;
use hemlock_ecs::resources::Resources;
use hemlock_ecs::storage::{Storage, spawn_with};
use hemlock_ecs::system::{IntoSystem, Mut, System, each_mut};

#[derive(Clone, Copy)]
struct A(f32);

#[derive(Clone, Copy)]
struct B(f32);

#[derive(Clone, Copy)]
struct C(f32);

#[derive(Clone, Copy)]
struct D(f32);

#[derive(Clone, Copy)]
struct E(f32);

fn ab(mut rows: Mut<(A, B)>) {
    each_mut(&mut rows, |_entity, (mut a, mut b), ()| {
        std::mem::swap(&mut a.0, &mut b.0);
    });
}

fn cd(mut rows: Mut<(C, D)>) {
    each_mut(&mut rows, |_entity, (mut c, mut d), ()| {
        std::mem::swap(&mut c.0, &mut d.0);
    });
}

fn ce(mut rows: Mut<(C, E)>) {
    each_mut(&mut rows, |_entity, (mut c, mut e), ()| {
        std::mem::swap(&mut c.0, &mut e.0);
    });
}

pub struct Benchmark(Storage, Resources, Vec<Vec<System>>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut storage = Storage::default();

        for _ in 0..10_000 {
            spawn_with(&mut storage, (A(0.0), B(0.0)));
        }
        for _ in 0..10_000 {
            spawn_with(&mut storage, (A(0.0), B(0.0), C(0.0)));
        }
        for _ in 0..10_000 {
            spawn_with(&mut storage, (A(0.0), B(0.0), C(0.0), D(0.0)));
        }
        for _ in 0..10_000 {
            spawn_with(&mut storage, (A(0.0), B(0.0), C(0.0), E(0.0)));
        }

        let mut systems = vec![ab.into_system(), cd.into_system(), ce.into_system()];
        let batches = batch(&mut systems, &[]);

        Self(storage, Resources::default(), batches)
    }

    pub fn run(&mut self) {
        hemlock_ecs::schedule::run(&mut self.2, &mut self.0, &mut self.1, &mut None, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemlock_ecs::storage::{living, query};

    #[test]
    fn schedule_covers_all_archetypes() {
        let mut bench = Benchmark::setup();
        assert_eq!(living(&bench.0).len(), 40_000);
        assert_eq!(query::<(&A, &B)>(&bench.0).count(), 40_000);
        assert_eq!(query::<(&C, &D)>(&bench.0).count(), 10_000);

        bench.run();
    }

    #[test]
    fn conflicting_systems_run_in_separate_batches() {
        let bench = Benchmark::setup();
        let sizes: Vec<usize> = bench.2.iter().map(Vec::len).collect();
        assert_eq!(sizes, vec![2, 1]);
    }
}
