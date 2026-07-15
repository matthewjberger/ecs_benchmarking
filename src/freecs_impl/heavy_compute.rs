use cgmath::*;
use freecs::ecs;

ecs! {
    World {
        matrix: Matrix => MATRIX,
        position: Position => POSITION,
        rotation: Rotation => ROTATION,
        velocity: Velocity => VELOCITY,
    }
    Resources {}
}

#[derive(Clone, Copy)]
pub struct Matrix(pub Matrix4<f32>);

impl Default for Matrix {
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
            MATRIX | POSITION | ROTATION | VELOCITY,
            1000,
            |table, idx| {
                table.matrix[idx] = Matrix(Matrix4::from_angle_x(Rad(1.2)));
                table.position[idx] = Position(Vector3::unit_x());
                table.rotation[idx] = Rotation(Vector3::unit_x());
                table.velocity[idx] = Velocity(Vector3::unit_x());
            },
        );

        Self(world)
    }

    pub fn run(&mut self) {
        self.0
            .par_for_each_mut(MATRIX | POSITION, 0, |_entity, table, idx| {
                for _ in 0..100 {
                    table.matrix[idx].0 = table.matrix[idx].0.invert().unwrap();
                }
                table.position[idx].0 = table.matrix[idx].0.transform_vector(table.position[idx].0);
            });
    }
}
