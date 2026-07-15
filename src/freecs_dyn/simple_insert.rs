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

pub struct Benchmark;

impl Benchmark {
    pub fn setup() -> Self {
        Self
    }

    pub fn run(&mut self) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_spawn_creates_all_entities() {
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
        assert_eq!(world.entity_count(), 10_000);

        let mut count = 0;
        world.query::<&Position>().for_each(|_entity, _| count += 1);
        assert_eq!(count, 10_000);
    }
}
