use freecs::dynamic::DynWorld;

#[derive(Clone, Copy, Default)]
struct Data(f32);

macro_rules! markers {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy, Default)]
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
            $world.spawn_bundles(($name, Data(1.0)), 20);
        )*
    };
}

pub struct Benchmark(DynWorld);

impl Benchmark {
    pub fn setup() -> Self {
        let mut world = DynWorld::new();

        spawn_markers!(
            world; V0, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16, V17,
            V18, V19, V20, V21, V22, V23, V24, V25,
        );

        Self(world)
    }

    pub fn run(&mut self) {
        self.0.query::<&mut Data>().for_each(|_entity, data| {
            data.0 *= 2.0;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_all_fragmented_archetypes() {
        let mut bench = Benchmark::setup();
        assert_eq!(bench.0.entity_count(), 520);

        bench.run();

        let mut count = 0;
        let mut last = 0.0;
        bench.0.query::<&Data>().for_each(|_entity, data| {
            count += 1;
            last = data.0;
        });
        assert_eq!(count, 520);
        assert!((last - 2.0).abs() < 1e-3);
    }
}
