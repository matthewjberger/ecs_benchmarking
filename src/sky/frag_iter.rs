use sky_ecs::World;

#[derive(Clone, Copy)]
struct Data(f32);

macro_rules! markers {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy)]
            struct $name;
        )*
    };
}

markers!(
    V0, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16, V17, V18, V19, V20,
    V21, V22, V23, V24, V25,
);

macro_rules! spawn_markers {
    ($world:ident; $($name:ident),* $(,)?) => {
        $(
            $world.spawn_batch((0..20).map(|_| ($name, Data(1.0))));
        )*
    };
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::new();

        spawn_markers!(
            world; V0, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16, V17,
            V18, V19, V20, V21, V22, V23, V24, V25,
        );

        Self(world)
    }

    pub fn run(&mut self) {
        self.0.query_mut::<&mut Data>().for_each(|data| {
            data.0 *= 2.0;
        });
    }
}
