use sky_ecs::{EntityId, World};

#[derive(Clone, Copy)]
struct A(f32);

#[derive(Clone, Copy)]
struct B(f32);

pub struct Benchmark(World, Vec<EntityId>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::new();
        let entities = (0..10_000).map(|_| world.spawn((A(0.0),))).collect();

        Self(world, entities)
    }

    pub fn run(&mut self) {
        for &entity in &self.1 {
            self.0.insert(entity, B(0.0));
        }

        for &entity in &self.1 {
            self.0.remove::<B>(entity);
        }
    }
}
