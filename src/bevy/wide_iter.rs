use bevy_ecs::prelude::*;

macro_rules! components {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Component, Clone, Copy)]
            struct $name(f32);
        )*
    };
}

components!(C0, C1, C2, C3, C4, C5, C6, C7);

pub const ENTITY_COUNT: usize = 8_000_000;

pub struct Benchmark(World);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::new();
        world.spawn_batch((0..ENTITY_COUNT).map(|_| {
            (
                C0(0.0),
                C1(1.0),
                C2(1.0),
                C3(1.0),
                C4(1.0),
                C5(1.0),
                C6(1.0),
                C7(1.0),
            )
        }));

        Self(world)
    }

    pub fn run(&mut self) {
        let mut query = self
            .0
            .query::<(&mut C0, &C1, &C2, &C3, &C4, &C5, &C6, &C7)>();

        for (mut c0, c1, c2, c3, c4, c5, c6, c7) in query.iter_mut(&mut self.0) {
            c0.0 += c1.0 + c2.0 + c3.0 + c4.0 + c5.0 + c6.0 + c7.0;
        }
    }
}
