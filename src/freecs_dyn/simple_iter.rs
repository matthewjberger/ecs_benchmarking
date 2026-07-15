use cgmath::*;
use freecs::dynamic::DynWorld;

#[derive(Clone, Copy)]
struct Transform(Matrix4<f32>);
impl Default for Transform {
    fn default() -> Self {
        Self(Matrix4::identity())
    }
}

#[derive(Clone, Copy)]
struct Position(Vector3<f32>);
impl Default for Position {
    fn default() -> Self {
        Self(Vector3::new(0.0, 0.0, 0.0))
    }
}

#[derive(Clone, Copy)]
struct Rotation(Vector3<f32>);
impl Default for Rotation {
    fn default() -> Self {
        Self(Vector3::new(0.0, 0.0, 0.0))
    }
}

#[derive(Clone, Copy)]
struct Velocity(Vector3<f32>);
impl Default for Velocity {
    fn default() -> Self {
        Self(Vector3::new(0.0, 0.0, 0.0))
    }
}

pub struct Benchmark(DynWorld);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = DynWorld::new();
        world.spawn_bundles(
            (
                Transform(Matrix4::from_scale(1.0)),
                Position(Vector3::unit_x()),
                Rotation(Vector3::unit_x()),
                Velocity(Vector3::unit_x()),
            ),
            10_000,
        );

        Self(world)
    }

    pub fn run(&mut self) {
        self.0
            .query::<(&Velocity, &mut Position)>()
            .for_each(|_entity, (velocity, position)| {
                position.0 += velocity.0;
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterates_all_entities_and_mutates() {
        let mut bench = Benchmark::setup();
        assert_eq!(bench.0.entity_count(), 10_000);

        bench.run();

        let mut count = 0;
        let mut sample_x = 0.0;
        bench.0.query::<&Position>().for_each(|_entity, position| {
            count += 1;
            sample_x = position.0.x;
        });
        assert_eq!(count, 10_000);
        assert!((sample_x - 2.0).abs() < 1e-3);
    }
}
