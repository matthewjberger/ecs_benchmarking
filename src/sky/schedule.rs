use sky_ecs::{Update, View, World};

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

fn ab(entities: View<(&mut A, &mut B)>) {
    entities.for_each(|(a, b)| std::mem::swap(&mut a.0, &mut b.0));
}

fn cd(entities: View<(&mut C, &mut D)>) {
    entities.for_each(|(c, d)| std::mem::swap(&mut c.0, &mut d.0));
}

fn ce(entities: View<(&mut C, &mut E)>) {
    entities.for_each(|(c, e)| std::mem::swap(&mut c.0, &mut e.0));
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::new();

        world.spawn_batch((0..10_000).map(|_| (A(0.0), B(0.0))));
        world.spawn_batch((0..10_000).map(|_| (A(0.0), B(0.0), C(0.0))));
        world.spawn_batch((0..10_000).map(|_| (A(0.0), B(0.0), C(0.0), D(0.0))));
        world.spawn_batch((0..10_000).map(|_| (A(0.0), B(0.0), C(0.0), E(0.0))));

        world.stage(Update).add(ab).add(cd).add(ce);

        Self(world)
    }

    pub fn run(&mut self) {
        self.0.tick_with_delta(0.0).unwrap();
    }
}
