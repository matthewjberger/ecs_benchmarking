use freecs::{Entity, ecs};

ecs! {
    World {
        a: A => A_COMP,
        b: B => B_COMP,
    }
    Resources {}
}

#[derive(Clone, Copy, Default)]
pub struct A(pub f32);

#[derive(Clone, Copy, Default)]
pub struct B(pub f32);

pub struct Benchmark(World, Vec<Entity>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::default();
        let entities = world.spawn_batch(A_COMP, 10_000, |_table, _idx| {});

        Self(world, entities)
    }

    pub fn run(&mut self) {
        for &entity in &self.1 {
            self.0.add_components(entity, B_COMP);
        }

        for &entity in &self.1 {
            self.0.remove_components(entity, B_COMP);
        }
    }
}
