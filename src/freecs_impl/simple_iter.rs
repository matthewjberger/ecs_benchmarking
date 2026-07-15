use cgmath::*;
use freecs::ecs;

ecs! {
    World {
        transform: Transform => TRANSFORM,
        position: Position => POSITION,
        rotation: Rotation => ROTATION,
        velocity: Velocity => VELOCITY,
    }
    Resources {}
}

#[derive(Clone, Copy)]
pub struct Transform(pub Matrix4<f32>);

impl Default for Transform {
    fn default() -> Self {
        Self(Matrix4::identity())
    }
}

#[derive(Clone, Copy)]
pub struct Position(pub Vector3<f32>);

impl Default for Position {
    fn default() -> Self {
        Self(Vector3::new(0.0, 0.0, 0.0))
    }
}

#[derive(Clone, Copy)]
pub struct Rotation(pub Vector3<f32>);

impl Default for Rotation {
    fn default() -> Self {
        Self(Vector3::new(0.0, 0.0, 0.0))
    }
}

#[derive(Clone, Copy)]
pub struct Velocity(pub Vector3<f32>);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vector3::new(0.0, 0.0, 0.0))
    }
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::default();
        world.spawn_batch(
            TRANSFORM | POSITION | ROTATION | VELOCITY,
            10_000,
            |table, idx| {
                table.transform[idx] = Transform(Matrix4::from_scale(1.0));
                table.position[idx] = Position(Vector3::unit_x());
                table.rotation[idx] = Rotation(Vector3::unit_x());
                table.velocity[idx] = Velocity(Vector3::unit_x());
            },
        );

        Self(world)
    }

    pub fn run(&mut self) {
        self.0
            .query_mut()
            .with(POSITION | VELOCITY)
            .iter(|_entity, table, idx| {
                table.position[idx].0 += table.velocity[idx].0;
            });
    }
}
