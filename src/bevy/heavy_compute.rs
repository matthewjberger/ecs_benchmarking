use bevy_ecs::prelude::*;
use bevy_tasks::{ComputeTaskPool, TaskPool};
use cgmath::*;

#[derive(Component, Copy, Clone)]
struct Position(Vector3<f32>);

#[derive(Component, Copy, Clone)]
struct Rotation(Vector3<f32>);

#[derive(Component, Copy, Clone)]
struct Velocity(Vector3<f32>);

#[derive(Component, Copy, Clone)]
struct Matrix(Matrix4<f32>);

pub struct Benchmark(World);

impl Benchmark {
    pub fn setup() -> Self {
        ComputeTaskPool::get_or_init(TaskPool::default);

        let mut world = World::new();
        world.spawn_batch((0..1000).map(|_| {
            (
                Matrix(Matrix4::<f32>::from_angle_x(Rad(1.2))),
                Position(Vector3::unit_x()),
                Rotation(Vector3::unit_x()),
                Velocity(Vector3::unit_x()),
            )
        }));

        Self(world)
    }

    pub fn run(&mut self) {
        let mut query = self.0.query::<(&mut Position, &mut Matrix)>();

        query
            .par_iter_mut(&mut self.0)
            .for_each(|(mut position, mut matrix)| {
                for _ in 0..100 {
                    matrix.0 = matrix.0.invert().unwrap();
                }
                position.0 = matrix.0.transform_vector(position.0);
            });
    }
}
