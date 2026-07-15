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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_then_removes_on_every_entity() {
        let mut bench = Benchmark::setup();
        assert_eq!(bench.0.entity_count(), 10_000);

        for &entity in &bench.1 {
            bench.0.set(entity, B(0.0));
        }
        let mut with_b = 0;
        bench.0.query::<&B>().for_each(|_entity, _| with_b += 1);
        assert_eq!(with_b, 10_000);

        for &entity in &bench.1 {
            bench.0.remove::<B>(entity);
        }
        let mut still_b = 0;
        bench.0.query::<&B>().for_each(|_entity, _| still_b += 1);
        assert_eq!(still_b, 0);
    }
}
