use freecs::Entity;
use freecs::dynamic::DynWorld;

#[derive(Clone, Copy, Default)]
struct A(f32);

#[derive(Clone, Copy, Default)]
struct B(f32);

pub struct Benchmark(DynWorld, Vec<Entity>);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = DynWorld::new();
        let entities = (0..10_000).map(|_| world.spawn((A(0.0),))).collect();

        Self(world, entities)
    }

    pub fn run(&mut self) {
        for &entity in &self.1 {
            self.0.set(entity, B(0.0));
        }

        for &entity in &self.1 {
            self.0.remove::<B>(entity);
        }
    }
}
