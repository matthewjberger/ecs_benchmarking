use freecs::ecs;

ecs! {
    World {
        a: A => A_COMP,
        b: B => B_COMP,
        c: C => C_COMP,
        d: D => D_COMP,
        e: E => E_COMP,
    }
    Resources {}
}

#[derive(Clone, Copy, Default)]
pub struct A(pub f32);

#[derive(Clone, Copy, Default)]
pub struct B(pub f32);

#[derive(Clone, Copy, Default)]
pub struct C(pub f32);

#[derive(Clone, Copy, Default)]
pub struct D(pub f32);

#[derive(Clone, Copy, Default)]
pub struct E(pub f32);

pub struct Benchmark(World, freecs::Schedule<World>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::default();

        world.spawn_batch(A_COMP | B_COMP, 10_000, |_table, _idx| {});
        world.spawn_batch(A_COMP | B_COMP | C_COMP, 10_000, |_table, _idx| {});
        world.spawn_batch(A_COMP | B_COMP | C_COMP | D_COMP, 10_000, |_table, _idx| {});
        world.spawn_batch(A_COMP | B_COMP | C_COMP | E_COMP, 10_000, |_table, _idx| {});

        let mut schedule = freecs::Schedule::<World>::new();

        schedule.push("ab", |world: &mut World| {
            world
                .query_mut()
                .with(A_COMP | B_COMP)
                .iter(|_entity, table, idx| {
                    std::mem::swap(&mut table.a[idx].0, &mut table.b[idx].0);
                });
        });

        schedule.push("cd", |world: &mut World| {
            world
                .query_mut()
                .with(C_COMP | D_COMP)
                .iter(|_entity, table, idx| {
                    std::mem::swap(&mut table.c[idx].0, &mut table.d[idx].0);
                });
        });

        schedule.push("ce", |world: &mut World| {
            world
                .query_mut()
                .with(C_COMP | E_COMP)
                .iter(|_entity, table, idx| {
                    std::mem::swap(&mut table.c[idx].0, &mut table.e[idx].0);
                });
        });

        Self(world, schedule)
    }

    pub fn run(&mut self) {
        self.1.run(&mut self.0);
    }
}
