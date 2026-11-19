use hemlock_ecs::later::{Edits, Later, despawn, remove, set, spawn};
use hemlock_ecs::resources::Resources;
use hemlock_ecs::storage::{Storage, spawn_batch};
use hemlock_ecs::system::{IntoSystem, System, View, each};

#[derive(Clone, Copy)]
struct Payload(f32);

#[derive(Clone, Copy)]
struct Transient;

#[derive(Clone, Copy)]
struct Tagged;

#[derive(Clone, Copy)]
struct Extra(f32);

pub const EDITS_OF_EACH_KIND: usize = 1_000;

fn spawner(mut later: Later) {
    for _ in 0..EDITS_OF_EACH_KIND {
        spawn(&mut later, (Transient, Payload(0.0)));
    }
}

fn attach(tagged: View<&Tagged>, mut edits: Edits) {
    for (entity, _tagged) in each(&tagged) {
        set(&mut edits, entity, Extra(0.0));
    }
}

fn detach(extras: View<&Extra>, mut edits: Edits) {
    for (entity, _extra) in each(&extras) {
        remove::<Extra>(&mut edits, entity);
    }
}

fn reaper(transients: View<&Transient>, mut edits: Edits) {
    for (entity, _transient) in each(&transients) {
        despawn(&mut edits, entity);
    }
}

pub struct Benchmark(Storage, Resources, Vec<Vec<System>>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut storage = Storage::default();
        spawn_batch(
            &mut storage,
            (0..EDITS_OF_EACH_KIND).map(|_| (Tagged, Payload(0.0))),
        );
        let batches = vec![
            vec![spawner.into_system(), attach.into_system()],
            vec![detach.into_system(), reaper.into_system()],
        ];

        let mut bench = Self(storage, Resources::default(), batches);
        bench.run();
        bench.run();
        bench
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
    fn each_run_leaves_the_storage_as_it_was() {
        let mut bench = Benchmark::setup();

        bench.run();

        assert_eq!(living(&bench.0).len(), EDITS_OF_EACH_KIND);
        assert_eq!(query::<&Tagged>(&bench.0).count(), EDITS_OF_EACH_KIND);
        assert_eq!(query::<&Transient>(&bench.0).count(), 0);
        assert_eq!(query::<&Extra>(&bench.0).count(), 0);
    }

    #[test]
    fn edits_land_between_the_two_batches() {
        let mut bench = Benchmark::setup();
        let (first, second) = bench.2.split_at_mut(1);

        hemlock_ecs::schedule::run(first, &mut bench.0, &mut bench.1, &mut None, false);
        assert_eq!(query::<&Transient>(&bench.0).count(), EDITS_OF_EACH_KIND);
        assert_eq!(query::<&Extra>(&bench.0).count(), EDITS_OF_EACH_KIND);

        hemlock_ecs::schedule::run(second, &mut bench.0, &mut bench.1, &mut None, false);
        assert_eq!(query::<&Transient>(&bench.0).count(), 0);
        assert_eq!(query::<&Extra>(&bench.0).count(), 0);
    }
}
