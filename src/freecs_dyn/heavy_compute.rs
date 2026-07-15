use cgmath::*;
use freecs::dynamic::DynWorld;

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

#[derive(Clone, Copy)]
struct Matrix(Matrix4<f32>);
impl Default for Matrix {
    fn default() -> Self {
        Self(Matrix4::identity())
    }
}

pub struct Benchmark(DynWorld);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = DynWorld::new();
        world.spawn_bundles(
            (
                Matrix(Matrix4::from_angle_x(Rad(1.2))),
                Position(Vector3::unit_x()),
                Rotation(Vector3::unit_x()),
                Velocity(Vector3::unit_x()),
            ),
            1000,
        );

        Self(world)
    }

    pub fn run(&mut self) {
        self.0.query::<(&mut Position, &mut Matrix)>().par_for_each(
            |_entity, (position, matrix)| {
                for _ in 0..100 {
                    matrix.0 = matrix.0.invert().unwrap();
                }
                position.0 = matrix.0.transform_vector(position.0);
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_all_entities_in_parallel() {
        let mut bench = Benchmark::setup();
        assert_eq!(bench.0.entity_count(), 1000);

        bench.run();

        let mut count = 0;
        bench
            .0
            .query::<&Position>()
            .for_each(|_entity, _| count += 1);
        assert_eq!(count, 1000);
    }
}
