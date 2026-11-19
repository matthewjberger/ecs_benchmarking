use hemlock_ecs::storage::{Storage, each_mut, spawn_with};

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
    ($storage:ident; $($name:ident),* $(,)?) => {
        $(
            for _ in 0..20 {
                spawn_with(&mut $storage, ($name, Data(1.0)));
            }
        )*
    };
}

pub struct Benchmark(Storage);

impl Benchmark {
    pub fn setup() -> Self {
        let mut storage = Storage::default();

        spawn_markers!(
            storage; V0, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16,
            V17, V18, V19, V20, V21, V22, V23, V24, V25,
        );

        Self(storage)
    }

    pub fn run(&mut self) {
        each_mut::<(Data,), ()>(&mut self.0, |_entity, (mut data,), ()| {
            data.0 *= 2.0;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemlock_ecs::storage::{living, query};

    #[test]
    fn processes_all_fragmented_archetypes() {
        let mut bench = Benchmark::setup();
        assert_eq!(living(&bench.0).len(), 520);

        bench.run();

        let mut count = 0;
        let mut last = 0.0;
        for (_entity, data) in query::<&Data>(&bench.0) {
            count += 1;
            last = data.0;
        }
        assert_eq!(count, 520);
        assert!((last - 2.0).abs() < 1e-3);
    }
}
