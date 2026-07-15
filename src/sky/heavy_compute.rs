use cgmath::*;
use sky_ecs::{ParView, Update, World};

#[derive(Clone, Copy)]
struct Position(Vector3<f32>);

#[derive(Clone, Copy)]
struct Rotation(Vector3<f32>);

#[derive(Clone, Copy)]
struct Velocity(Vector3<f32>);

#[derive(Clone, Copy)]
struct Matrix(Matrix4<f32>);

fn compute(bodies: ParView<(&mut Position, &mut Matrix)>) {
    bodies.par_for_each(|(position, matrix)| {
        for _ in 0..100 {
            matrix.0 = matrix.0.invert().unwrap();
        }
        position.0 = matrix.0.transform_vector(position.0);
    });
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::new();
        world.spawn_batch((0..1000).map(|_| {
            (
                Matrix(Matrix4::from_angle_x(Rad(1.2))),
                Position(Vector3::unit_x()),
                Rotation(Vector3::unit_x()),
                Velocity(Vector3::unit_x()),
            )
        }));
        world.stage(Update).add(compute);

        Self(world)
    }

    pub fn run(&mut self) {
        self.0.tick_with_delta(0.0).unwrap();
    }
}
