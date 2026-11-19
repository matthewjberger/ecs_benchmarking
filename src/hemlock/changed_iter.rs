use cgmath::*;
use hemlock_ecs::order::batch;
use hemlock_ecs::resources::{Resources, insert};
use hemlock_ecs::storage::{Storage, spawn_batch};
use hemlock_ecs::system::{IntoSystem, Mut, Peek, ResMut, System, changed, each_mut, ticked};

#[derive(Clone, Copy)]
struct Position(Vector3<f32>);

#[derive(Clone, Copy)]
struct Velocity(Vector3<f32>);

#[derive(Default)]
struct Gathered {
    since: u64,
    moved: usize,
}

pub const ENTITY_COUNT: usize = 10_000;
pub const MOVING_EVERY: usize = 10;

fn integrate(mut bodies: Mut<(Position,), (&Velocity,)>) {
    each_mut(&mut bodies, |_entity, (mut position,), (velocity,)| {
        if velocity.0.x != 0.0 {
            position.0 += velocity.0;
        }
    });
}

fn gather(positions: Peek<Position>, mut gathered: ResMut<Gathered>) {
    let since = gathered.since;
    gathered.since = ticked(&positions);
    gathered.moved = changed(&positions, since).len();
}

pub struct Benchmark(Storage, Resources, Vec<Vec<System>>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut storage = Storage::default();
        spawn_batch(
            &mut storage,
            (0..ENTITY_COUNT).map(|index| {
                let speed = match index % MOVING_EVERY {
                    0 => 1.0,
                    _ => 0.0,
                };
                (
                    Position(Vector3::unit_x()),
                    Velocity(Vector3::new(speed, 0.0, 0.0)),
                )
            }),
        );
        let mut resources = Resources::default();
        insert(&mut resources, Gathered::default());
        let mut systems = vec![integrate.into_system(), gather.into_system()];
        let batches = batch(&mut systems, &[]);

        let mut bench = Self(storage, resources, batches);
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
    use hemlock_ecs::resources::get;
    use hemlock_ecs::storage::query;

    #[test]
    fn gathers_only_the_rows_that_moved() {
        let mut bench = Benchmark::setup();

        bench.run();

        let gathered = get::<Gathered>(&bench.1).unwrap();
        assert_eq!(gathered.moved, ENTITY_COUNT / MOVING_EVERY);
        let farthest = query::<&Position>(&bench.0)
            .map(|(_entity, position)| position.0.x)
            .fold(0.0, f32::max);
        assert!((farthest - 4.0).abs() < 1e-3);
    }

    #[test]
    fn writer_and_reader_run_in_separate_batches() {
        let bench = Benchmark::setup();
        let sizes: Vec<usize> = bench.2.iter().map(Vec::len).collect();
        assert_eq!(sizes, vec![1, 1]);
    }
}
