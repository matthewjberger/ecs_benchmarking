use freecs::Schedule;
use freecs::dynamic::DynWorld;

#[derive(Clone, Copy, Default)]
struct A(f32);

#[derive(Clone, Copy, Default)]
struct B(f32);

#[derive(Clone, Copy, Default)]
struct C(f32);

#[derive(Clone, Copy, Default)]
struct D(f32);

#[derive(Clone, Copy, Default)]
struct E(f32);

pub struct Benchmark(DynWorld, Schedule<DynWorld>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = DynWorld::new();

        world.spawn_bundles((A(0.0), B(0.0)), 10_000);
        world.spawn_bundles((A(0.0), B(0.0), C(0.0)), 10_000);
        world.spawn_bundles((A(0.0), B(0.0), C(0.0), D(0.0)), 10_000);
        world.spawn_bundles((A(0.0), B(0.0), C(0.0), E(0.0)), 10_000);

        let mut schedule = Schedule::<DynWorld>::new();

        schedule.push("ab", |world: &mut DynWorld| {
            world
                .query::<(&mut A, &mut B)>()
                .for_each(|_entity, (a, b)| std::mem::swap(&mut a.0, &mut b.0));
        });

        schedule.push("cd", |world: &mut DynWorld| {
            world
                .query::<(&mut C, &mut D)>()
                .for_each(|_entity, (c, d)| std::mem::swap(&mut c.0, &mut d.0));
        });

        schedule.push("ce", |world: &mut DynWorld| {
            world
                .query::<(&mut C, &mut E)>()
                .for_each(|_entity, (c, e)| std::mem::swap(&mut c.0, &mut e.0));
        });

        Self(world, schedule)
    }

    pub fn run(&mut self) {
        self.1.run(&mut self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_covers_all_archetypes() {
        let mut bench = Benchmark::setup();
        assert_eq!(bench.0.entity_count(), 40_000);

        let mut ab = 0;
        bench.0.query::<(&A, &B)>().for_each(|_entity, _| ab += 1);
        assert_eq!(ab, 40_000);
        let mut cd = 0;
        bench.0.query::<(&C, &D)>().for_each(|_entity, _| cd += 1);
        assert_eq!(cd, 10_000);

        bench.run();
    }
}
