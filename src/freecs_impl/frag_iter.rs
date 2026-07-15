use freecs::ecs;

ecs! {
    World {
        data: Data => DATA,
        v0: V0 => M0,
        v1: V1 => M1,
        v2: V2 => M2,
        v3: V3 => M3,
        v4: V4 => M4,
        v5: V5 => M5,
        v6: V6 => M6,
        v7: V7 => M7,
        v8: V8 => M8,
        v9: V9 => M9,
        v10: V10 => M10,
        v11: V11 => M11,
        v12: V12 => M12,
        v13: V13 => M13,
        v14: V14 => M14,
        v15: V15 => M15,
        v16: V16 => M16,
        v17: V17 => M17,
        v18: V18 => M18,
        v19: V19 => M19,
        v20: V20 => M20,
        v21: V21 => M21,
        v22: V22 => M22,
        v23: V23 => M23,
        v24: V24 => M24,
        v25: V25 => M25,
    }
    Resources {}
}

#[derive(Clone, Copy, Default)]
pub struct Data(pub f32);

macro_rules! markers {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy, Default)]
            pub struct $name(pub f32);
        )*
    };
}

markers!(
    V0, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16, V17, V18, V19, V20,
    V21, V22, V23, V24, V25,
);

pub struct Benchmark(World);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = World::default();

        let masks = [
            M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12, M13, M14, M15, M16, M17, M18,
            M19, M20, M21, M22, M23, M24, M25,
        ];

        for mask in masks {
            world.spawn_batch(mask | DATA, 20, |table, idx| {
                table.data[idx] = Data(1.0);
            });
        }

        Self(world)
    }

    pub fn run(&mut self) {
        self.0.query_mut().with(DATA).iter(|_entity, table, idx| {
            table.data[idx].0 *= 2.0;
        });
    }
}
